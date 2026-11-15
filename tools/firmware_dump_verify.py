#!/usr/bin/env python3
# 固件只读 dump 校验工具
# 用途：解析 UART 串口日志里的 U-Boot `md` 十六进制输出，重建二进制分区镜像，
#       检测串口丢行造成的缺口，计算 MD5/SHA256，并可与官方 .img 对应区间逐字节比对。
# 约束：本脚本只处理文本日志与本地文件，不接触设备、不执行任何写 flash 操作。

import argparse
import hashlib
import re
import sys
import zlib
from pathlib import Path

# 匹配 md 输出行：可选 0x 前缀的地址 + 冒号 + 十六进制字节
ADDR_RE = re.compile(r"^\s*(?:0x)?([0-9a-fA-F]{8,16})\s*:\s*(.*)$")
HEX2 = re.compile(r"^[0-9a-fA-F]{2}$")


def parse_logs(paths):
    """解析多份日志，返回 {行起始地址: 该行字节}；同一地址重复出现时以最后一次为准"""
    rows = {}
    total_lines = 0
    for p in paths:
        try:
            with open(p, "r", encoding="utf-8", errors="ignore") as f:
                for line in f:
                    m = ADDR_RE.match(line)
                    if not m:
                        continue
                    addr = int(m.group(1), 16)
                    vals = []
                    # 逐个取冒号后的 2 位十六进制，遇到 ASCII 注释等非字节内容即停止
                    for tok in m.group(2).split():
                        if not HEX2.match(tok):
                            break
                        vals.append(int(tok, 16))
                    if vals:
                        rows[addr] = bytes(vals)
                        total_lines += 1
        except OSError as e:
            print(f"[错误] 无法读取日志 {p}: {e}", file=sys.stderr)
    return rows, total_lines


def rebuild(rows, base):
    """按地址重建连续缓冲区；未覆盖区域填 0xFF 并返回存在掩码"""
    if not rows:
        raise SystemExit("[错误] 日志中没有解析到任何 md 十六进制行")
    if base is None:
        base = min(rows)
    end = max(addr + len(b) for addr, b in rows.items())
    if end <= base:
        raise SystemExit(f"[错误] 地址范围异常：base=0x{base:X} end=0x{end:X}")
    size = end - base
    buf = bytearray(b"\xff" * size)
    mask = bytearray(size)
    for addr, b in rows.items():
        off = addr - base
        if off < 0:
            continue
        buf[off:off + len(b)] = b
        mask[off:off + len(b)] = b"\x01" * len(b)
    return base, buf, mask


def gap_ranges(mask, limit=10):
    """返回连续缺失区间（起始偏移, 长度），最多 limit 段"""
    out = []
    i = 0
    n = len(mask)
    while i < n:
        if mask[i]:
            i += 1
            continue
        j = i
        while j < n and not mask[j]:
            j += 1
        out.append((i, j - i))
        if len(out) >= limit:
            break
        i = j
    return out


def digest(name, data):
    if name == "md5":
        return hashlib.md5(data).hexdigest()
    return hashlib.sha256(data).hexdigest()


def main():
    ap = argparse.ArgumentParser(
        description="从 UART 日志重建固件分区并校验（只读，不接触设备）")
    ap.add_argument("logs", nargs="+", help="串口日志文件（可多个）")
    ap.add_argument("--base", type=lambda s: int(s, 0), default=None,
                    help="md 输出的起始 RAM 地址，如 0x42000000；默认取日志最小地址")
    ap.add_argument("--flash-offset", type=lambda s: int(s, 0), default=None,
                    help="该段在 flash 中的偏移，如 rodata=0x2C0000、data=0x5C0000")
    ap.add_argument("--output", default=None, help="重建后的 .bin 输出路径")
    ap.add_argument("--expect-md5", default=None, help="期望的 MD5，用于校验")
    ap.add_argument("--expect-sha256", default=None, help="期望的 SHA-256，用于校验")
    ap.add_argument("--compare", default=None,
                    help="与官方 .img 的 --flash-offset 区间逐字节比对")
    ap.add_argument("--merge-image", default=None,
                    help="把重建结果合并进官方 .img 的副本（需配合 --flash-offset）")
    ap.add_argument("--merge-out", default=None, help="合并后的输出文件路径")
    args = ap.parse_args()

    rows, lines = parse_logs(args.logs)
    print(f"[信息] 解析日志行 {lines} 条，唯一地址行 {len(rows)} 条")
    base, buf, mask = rebuild(rows, args.base)
    print(f"[信息] 重建区间：RAM 0x{base:X} - 0x{base + len(buf):X}，长度 {len(buf)} 字节")

    missing = len(mask) - sum(mask)
    if missing:
        print(f"[警告] 缺失 {missing} 字节（{missing / len(mask) * 100:.2f}%），可能存在串口丢行：")
        for off, length in gap_ranges(mask):
            print(f"        RAM 0x{base + off:X} 起缺失 {length} 字节", end="")
            if args.flash_offset is not None:
                print(f"（flash 0x{args.flash_offset + off:X}）")
            else:
                print()
        print("[建议] 对缺失区间重新执行 md 并追加日志后再次运行本脚本")
    else:
        print("[信息] 覆盖完整，无缺口")

    md5 = digest("md5", buf)
    sha = digest("sha256", buf)
    crc = zlib.crc32(buf) & 0xFFFFFFFF
    print(f"[校验] MD5    {md5}")
    print(f"[校验] SHA256 {sha}")
    print(f"[校验] CRC32  0x{crc:08X}（可与 U-Boot 的 crc32 结果对照）")

    if args.expect_md5:
        ok = md5.lower() == args.expect_md5.lower()
        print(f"[校验] MD5 期望值比对：{'通过' if ok else '不一致'}")
    if args.expect_sha256:
        ok = sha.lower() == args.expect_sha256.lower()
        print(f"[校验] SHA-256 期望值比对：{'通过' if ok else '不一致'}")

    if args.compare:
        img = Path(args.compare)
        if args.flash_offset is None:
            raise SystemExit("[错误] --compare 需要同时指定 --flash-offset")
        raw = img.read_bytes()
        start = args.flash_offset
        if start + len(buf) > len(raw):
            print(f"[错误] 官方镜像长度不足：需要 0x{start + len(buf):X}，实际 0x{len(raw):X}")
        else:
            diff = 0
            first = None
            for i in range(len(buf)):
                if not mask[i]:
                    continue
                if raw[start + i] != buf[i]:
                    diff += 1
                    if first is None:
                        first = i
            print(f"[比对] 与官方镜像差异字节 {diff}", end="")
            if first is not None:
                print(f"，首个差异在 flash 0x{start + first:X}（dump=0x{buf[first]:02X} img=0x{raw[start + first]:02X}）")
            else:
                print("（完全一致）")

    if args.output:
        Path(args.output).write_bytes(buf)
        print(f"[输出] 已写入 {args.output}")

    if args.merge_image and args.merge_out:
        if args.flash_offset is None:
            raise SystemExit("[错误] --merge-image 需要同时指定 --flash-offset")
        raw = bytearray(Path(args.merge_image).read_bytes())
        start = args.flash_offset
        if start + len(buf) > len(raw):
            raw.extend(b"\xff" * (start + len(buf) - len(raw)))
        raw[start:start + len(buf)] = buf
        Path(args.merge_out).write_bytes(raw)
        print(f"[输出] 已合并到 {args.merge_out}（原始镜像未修改）")


if __name__ == "__main__":
    main()
