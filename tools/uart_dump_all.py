#!/usr/bin/env python3
# N4 Pro 全分区只读 dump 自动化脚本
# 通过 UART 进入 U-Boot/tinySPL 控制台，逐块执行 spinor read + md，把 6 个分区完整读回。
# 安全约束：只发送白名单命令（spinor init/read、md、crc32、part dump、printenv），
#           绝不发送 erase/write/saveenv/run/ramboot/aicupg 等任何写操作。
# 输出：<out>/<分区>/<flash偏移>.bin 与 .log，最终合并为 <out>/<分区>.bin

import argparse
import re
import sys
import time
import zlib
from pathlib import Path

try:
    import serial
except ImportError:
    raise SystemExit("缺少 pyserial，请先执行：pip install pyserial")

# 官方分区表（来自固件 FAL 字符串）
PARTITIONS = [
    ("spl", 0x000000, 0x080000),
    ("env", 0x080000, 0x020000),
    ("env_r", 0x0A0000, 0x020000),
    ("os", 0x0C0000, 0x200000),
    ("rodata", 0x2C0000, 0x300000),
    ("data", 0x5C0000, 0x900000),
]

ADDR_RE = re.compile(r"^\s*(?:0x)?([0-9a-fA-F]{8,16})\s*:\s*(.*)$")
HEX2 = re.compile(r"^[0-9a-fA-F]{2}$")
CRC_RE = re.compile(r"(?:==>|:)\s*([0-9a-fA-F]{8})\b")

# 只允许这些命令前缀
ALLOWED = ("spinor init", "spinor read", "md", "crc32", "part dump", "part list", "printenv")


def send(ser, cmd, idle=1.2, max_wait=90):
    """发送一条白名单命令并读取到串口空闲"""
    if not cmd.startswith(ALLOWED):
        raise RuntimeError(f"拒绝发送非白名单命令：{cmd}")
    ser.reset_input_buffer()
    ser.write(cmd.encode("ascii") + b"\r\n")
    buf = bytearray()
    last = time.time()
    start = time.time()
    while time.time() - last < idle and time.time() - start < max_wait:
        n = ser.in_waiting
        if n:
            buf.extend(ser.read(n))
            last = time.time()
        else:
            time.sleep(0.03)
    return bytes(buf)


def parse_md(text):
    """解析 md 输出为 {起始地址: 字节串}"""
    rows = {}
    for line in text.splitlines():
        m = ADDR_RE.match(line)
        if not m:
            continue
        addr = int(m.group(1), 16)
        vals = []
        for tok in m.group(2).split():
            if not HEX2.match(tok):
                break
            vals.append(int(tok, 16))
        if vals:
            rows[addr] = bytes(vals)
    return rows


def find_prompt(ser):
    """尝试确认处于命令行（只发回车，不执行命令）"""
    ser.reset_input_buffer()
    ser.write(b"\r\n")
    time.sleep(0.8)
    data = ser.read(4096).decode("utf-8", errors="replace")
    hints = ("=>", "#", "U-Boot", "tinySPL", "Aic", "aicupg")
    return any(h in data for h in hints), data


def main():
    ap = argparse.ArgumentParser(description="N4 Pro 全分区只读 dump（UART）")
    ap.add_argument("--port", required=True, help="串口号，如 COM5")
    ap.add_argument("--baud", type=int, default=115200)
    ap.add_argument("--out", default="rodata", help="输出目录，默认 rodata")
    ap.add_argument("--ram", type=lambda s: int(s, 0), default=0x42000000,
                    help="spinor read 的 RAM 目标地址，默认 0x42000000")
    ap.add_argument("--chunk", type=lambda s: int(s, 0), default=0x10000,
                    help="每块大小，默认 0x10000(64KB)")
    ap.add_argument("--only", default=None, help="只处理指定分区，逗号分隔")
    ap.add_argument("--no-crc", action="store_true", help="关闭 CRC32 校验")
    args = ap.parse_args()

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    try:
        ser = serial.Serial(args.port, args.baud, timeout=0.2)
    except Exception as e:
        raise SystemExit(f"无法打开串口 {args.port}：{e}")

    ok, banner = find_prompt(ser)
    if not ok:
        print("[错误] 未检测到命令行提示。请确认：")
        print("       1) UART 已连接到设备且端口正确；")
        print("       2) 设备已重新上电，并在启动时连续回车进入控制台；")
        print("       3) 波特率正确（可试 1500000/921600/57600）。")
        print("       收到的内容：", banner[-300:])
        ser.close()
        return 2

    print("[信息] 已进入命令行，仅执行只读命令")
    print(send(ser, "part dump").decode("utf-8", errors="replace")[-1500:])
    print(send(ser, "spinor init 0").decode("utf-8", errors="replace")[-300:])

    todo = PARTITIONS
    if args.only:
        names = {x.strip() for x in args.only.split(",")}
        todo = [p for p in PARTITIONS if p[0] in names]

    for name, base, size in todo:
        pdir = out / name
        pdir.mkdir(parents=True, exist_ok=True)
        chunks = (size + args.chunk - 1) // args.chunk
        print(f"\n===== {name}  0x{base:06X}  0x{size:X}  共 {chunks} 块 =====")
        parts = []
        for i in range(chunks):
            off = i * args.chunk
            length = min(args.chunk, size - off)
            flash = base + off
            bin_path = pdir / f"{flash:08X}.bin"
            log_path = pdir / f"{flash:08X}.log"
            if bin_path.exists() and bin_path.stat().st_size == length:
                parts.append(bin_path)
                print(f"  [跳过] 0x{flash:06X} 已存在")
                continue

            ok_block = False
            for attempt in range(1, 4):
                out1 = send(ser, f"spinor read 0x{args.ram:X} 0x{flash:X} 0x{length:X}", idle=0.6, max_wait=30)
                md_out = send(ser, f"md 0x{args.ram:X} 0x{length:X}", idle=1.5, max_wait=120)
                rows = parse_md(md_out.decode("utf-8", errors="replace"))
                data = bytearray()
                addr = args.ram
                missing = 0
                for _ in range(length):
                    if addr in rows:
                        data.append(rows[addr][0])
                        addr += 1
                    else:
                        data.append(0xFF)
                        addr += 1
                        missing += 1

                # CRC 校验（可选）
                crc_ok = True
                if not args.no_crc:
                    crc_out = send(ser, f"crc32 0x{args.ram:X} 0x{length:X}", idle=0.8, max_wait=20)
                    m = CRC_RE.search(crc_out.decode("utf-8", errors="replace"))
                    if m:
                        expected = int(m.group(1), 16)
                        actual = zlib.crc32(bytes(data)) & 0xFFFFFFFF
                        crc_ok = expected == actual

                if missing == 0 and crc_ok:
                    bin_path.write_bytes(bytes(data))
                    log_path.write_bytes(out1 + b"\n" + md_out)
                    parts.append(bin_path)
                    print(f"  [完成] 0x{flash:06X} {length} 字节")
                    ok_block = True
                    break
                else:
                    print(f"  [重试 {attempt}/3] 0x{flash:06X} 缺失 {missing} 字节，CRC {'通过' if crc_ok else '不符'}")

            if not ok_block:
                print(f"  [失败] 0x{flash:06X} 连续 3 次未通过，停止。已完成的块可直接重跑续传。")
                ser.close()
                return 3

        merged = out / f"{name}.bin"
        with open(merged, "wb") as f:
            for p in parts:
                f.write(p.read_bytes())
        print(f"[合并] {merged}  {merged.stat().st_size} 字节")

    ser.close()

    # 汇总校验
    import hashlib
    print("\n===== 汇总 =====")
    for name, base, size in todo:
        p = out / f"{name}.bin"
        if p.exists():
            b = p.read_bytes()
            print(f"{name:8} {len(b):>9} 字节  MD5 {hashlib.md5(b).hexdigest()}  "
                  f"SHA256 {hashlib.sha256(b).hexdigest()[:32]}…  CRC32 0x{zlib.crc32(b) & 0xFFFFFFFF:08X}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
