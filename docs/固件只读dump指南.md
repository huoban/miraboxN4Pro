# N4 Pro 固件分区只读 dump 指南

> 目标：从设备 SPI NOR 读取 `rodata`（3 MB）与 `data`（9 MB）分区，用于提取图标、字库与协议表。
> 原则：**全程只读**。本文所有命令都不写入 flash；配套校验脚本见 `tools/firmware_dump_verify.py`。
> 依据：固件内 U-Boot/tinySPL 命令表（见 `docs/逆向固件包.md` 第 9 节）。

---

## 1. 安全红线

### 允许（只读）

| 命令 | 作用 |
|---|---|
| `spinor init 0` | 初始化 SPI NOR（仅使能设备） |
| `spinor read <addr> <offset> <size>` | 把 flash 读到 RAM |
| `md [b/w/l] <addr> <count>` | 显示 RAM 内容 |
| `md` 默认模式为 **b（字节）**，本流程只用默认模式 |
| `part list` / `part dump` | 查看分区表 |
| `mtd list` | 列出 MTD 设备 |
| `printenv` | 查看环境变量（只读） |
| `crc32 <addr> <length>` | 计算 RAM 数据 CRC32 |

### 禁止（会改写 flash 或环境）

| 命令 | 后果 |
|---|---|
| `spinor erase` / `spinor write` | 擦除/写入 flash，可能变砖 |
| `mtd erase` / `mtd write` | 同上 |
| `saveenv` / `fw_setenv` | 改写 env 分区 |
| `mtd bypass 0/1` | 切换 SPI 加密旁路，影响运行状态 |
| `run <cmd>` | 可执行任意命令，禁止使用 |
| `ramboot` | 从 RAM 启动，会改变运行流程 |
| `aicupg hid/usb/uart ...` | 进入升级/刷写流程，非 dump 所需 |

> 只要不执行“禁止”表里的命令，读取操作不会改变固件内容。

---

## 2. 前置准备

1. **串口**：USB-TTL 接设备 UART（3.3V，TX/RX/GND），或使用板载调试口。
2. **串口工具**：PuTTY / MobaXterm / SecureCRT，必须开启**日志记录到文件**，且不限制行数。
3. **供电**：稳定电源，dump 期间不要断电（只读断电不会变砖，但会中断）。
4. **波特率**：固件只说明 `Console UART ready`，未标注速率。优先尝试 **115200 8N1**；无输出再试 1500000、921600、57600。
5. **进入控制台**：上电后 1–3 秒内连续发送回车或 `Ctrl+C` 打断启动。若直接进入系统，需要重试；tinySPL 阶段可能先打印 `tinySPL [Built on ...]`，此时即可尝试输入。

> 如果无法进入控制台，改用 SPI 编程器夹子直接读整片 flash（零命令风险，需拆机）。

---

## 3. 确认分区与 RAM 地址（只读）

```text
part list
part dump
mtd list
printenv
```

预期分区（N4 Pro）：

```
spi0.0:512k(spl),128k(env),128k(env_r),2m(os),3m(rodata),9m(data)
```

| 分区 | flash 偏移 | 大小 |
|---|---|---|
| spl | 0x000000 | 0x80000 |
| env | 0x080000 | 0x20000 |
| env_r | 0x0A0000 | 0x20000 |
| os | 0x0C0000 | 0x200000 |
| rodata | 0x2C0000 | 0x300000 |
| data | 0x5C0000 | 0x900000 |

RAM 地址：先看 `printenv` 里是否有 `loadaddr`（如 `0x40000000`）。默认示例使用 `0x42000000`，若该地址不可用（读回全 0 或异常），改用 `loadaddr` 的值。

---

## 4. dump 命令序列（以 rodata 为例）

### 4.1 关键约束：每块使用递增 RAM 地址

`md` 日志里的地址是 RAM 地址，校验脚本按地址重建。**不要每块都用同一个 RAM 地址**，否则日志里地址重复、内容互相覆盖。做法：第 N 块读到 `0x42000000 + N×块大小`。

### 4.2 分块大小

建议 **64 KB/块**（`0x10000`）：一次 `md` 输出约 4096 行，串口不易丢行；如需更快可改 128 KB。

### 4.3 命令模板

```text
spinor init 0
spinor read 0x42000000 0x2C0000 0x10000
md 0x42000000 0x10000
spinor read 0x42010000 0x2D0000 0x10000
md 0x42010000 0x10000
spinor read 0x42020000 0x2E0000 0x10000
md 0x42020000 0x10000
...
```

- `spinor read` 参数顺序：`<RAM 地址> <flash 偏移> <长度>`。
- 每块 flash 偏移 +0x10000、RAM 地址 +0x10000，直到覆盖 `0x2C0000`–`0x5C0000`（共 48 块）。
- `data` 分区：`0x5C0000`–`0xEC0000`（9 MB，144 块），RAM 地址从 `0x42000000` 重新递增即可（另存一份日志）。
- 若 RAM 容量不足，改为每块单独一个日志文件，分别校验后合并。

### 4.4 建议先 dump 小范围验证

先只做第一块，用 `crc32` 对照，确认解析无误后再批量执行：

```text
spinor read 0x42000000 0x2C0000 0x10000
crc32 0x42000000 0x10000
md 0x42000000 0x10000
```

把 U-Boot 打印的 CRC32 与脚本算出的 CRC32 对比，两者一致说明日志完整。

---

## 5. 校验与重建

### 5.1 单段校验（rodata）

```text
python tools/firmware_dump_verify.py logs\rodata.log ^
  --base 0x42000000 ^
  --flash-offset 0x2C0000 ^
  --output docs\evidence\firmware\rodata.bin
```

输出内容：

- 日志解析行数、重建区间；
- 缺口报告（串口丢行会显示缺失的 RAM/flash 区间）；
- MD5、SHA-256、CRC32；
- 可选 `--expect-md5` / `--expect-sha256` 对照。

### 5.2 与官方镜像比对（可选）

对 `spl` / `os` 分区（官方 img 内包含）：

```text
python tools/firmware_dump_verify.py logs\os.log ^
  --base 0x42000000 --flash-offset 0xC0000 ^
  --compare "V4.N4 Pro_HJDF.02.015.img"
```

`rodata` / `data` 不在官方 img 内，`--compare` 会因镜像长度不足而报错，这是预期的。

### 5.3 合并进整片镜像（可选）

```text
python tools/firmware_dump_verify.py logs\rodata.log ^
  --base 0x42000000 --flash-offset 0x2C0000 ^
  --merge-image "V4.N4 Pro_HJDF.02.015.img" ^
  --merge-out "V4.N4 Pro_HJDF.02.015.full.img"
```

原始 img 不会被修改，只写新文件。

---

## 6. 常见问题

| 现象 | 原因 | 处理 |
|---|---|---|
| 脚本报“缺失 N 字节” | 串口丢行/终端截断 | 对缺失区间重新 `md`，追加到日志后重跑 |
| 读回全 `FF` | flash 偏移错误或分区为空 | 用 `part dump` 核对偏移；rodata 前段可能有空洞 |
| 读回全 `00` | RAM 地址不可用/未初始化 | 换 `loadaddr` 或其它 DDR 区域 |
| `spinor init` 失败 | SPI 控制器未就绪/参数错 | 重试 `spinor init 0`；确认控制器编号 |
| 无法进入控制台 | 启动过快/波特率不对 | 上电后连续回车；尝试其它波特率 |
| `md` 输出带 ASCII 尾部 | 正常，脚本会自动忽略非 2 位十六进制 | 无需处理 |

---

## 7. 记录模板

每次 dump 建议记录：

```
设备序列号：
固件版本：V4.N4 Pro_HJDF.02.015
分区：rodata / data
flash 偏移：0x2C0000 / 0x5C0000
块大小：0x10000
RAM 基址：0x42000000
日志文件：logs\rodata_YYYYMMDD.log
重建结果：rodata.bin
MD5：
SHA256：
CRC32：
缺口：无 / 区间列表
```

---

## 8. 时间预估

115200 8N1 下，`md` 输出约为数据量的 3 倍（每字节 2 位十六进制 + 空格）。理论速度约 11 KB/s：

| 分区 | 数据量 | 预计串口时间 |
|---|---|---|
| rodata | 3 MB | 约 12–15 分钟 |
| data | 9 MB | 约 35–45 分钟 |

建议先 dump `rodata`（字库/图标更可能在这里），确认有内容后再决定是否 dump 整个 `data`。
