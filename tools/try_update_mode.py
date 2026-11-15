# 受控探测：向 N4 Pro 厂商 HID 接口发送 UpdateMode 候选报文
# 安全说明：只发送 "CRT APPNEW" 更新模式请求，不执行任何 flash 写/擦命令。
#           设备可能重启进入 usbupg 升级模式；断电重连即可恢复。
import subprocess
import sys
import time
from pathlib import Path

import hid

VID, PID = 0x5548, 0x1008
UPD_DIR = Path(r"C:\Program Files (x86)\StreamDock\UpDateToolV3")
UPG = str(UPD_DIR / "upgcmdHid.exe")


def vendor_path():
    for d in hid.enumerate(VID, PID):
        if d["usage_page"] == 0xFFA0:
            return d["path"]
    return None


def upg_list():
    try:
        r = subprocess.run([UPG, "-l"], cwd=str(UPD_DIR), capture_output=True,
                           text=True, timeout=15)
        return (r.stdout + r.stderr).strip()
    except Exception as e:
        return f"(调用 upgcmdHid 失败: {e})"


def main():
    first = int(sys.argv[1], 0) if len(sys.argv) > 1 else 0x00
    p = vendor_path()
    if not p:
        print("[错误] 未找到 VID 0x5548/PID 0x1008 的厂商 HID 接口（usage_page 0xFFA0）")
        return 2

    cmd = b"CRT\x00\x00APPNEW"
    buf = bytearray(513)
    buf[0] = first
    buf[1:1 + len(cmd)] = cmd

    print(f"[信息] 接口 {p}")
    print(f"[信息] 发送 513 字节，首字节 0x{first:02X}，命令 {cmd!r}")
    try:
        h = hid.device()
        h.open_path(p)
        n = h.write(buf)
        h.close()
        print(f"[信息] hid_write 返回 {n}")
    except Exception as e:
        print(f"[警告] 写入异常（设备可能已开始重启）：{e}")

    print("[信息] 等待设备枚举…")
    for i in range(6):
        time.sleep(2)
        out = upg_list()
        print(f"  [{i + 1}] upgcmdHid -l: {out.splitlines()[0] if out else '(空)'}")
        if "No usbupg device" not in out and out:
            print("[成功] 检测到 usbupg 设备，可执行只读 dump")
            return 0
    print("[结果] 未检测到 usbupg 设备（该候选报文未触发或需要其它首字节）")
    return 1


if __name__ == "__main__":
    sys.exit(main())
