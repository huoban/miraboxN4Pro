# default.img 逆向分析（UpDateToolV3 内置默认固件）

> 样本：`C:\Program Files (x86)\StreamDock\UpDateToolV3\default.img`
> 结论先行：**不是 N4 Pro 的 factory 区，不能刷入 N4 Pro**。它是 ArtInChip **D6 开发板**的完整演示/测试固件。

---

## 1. 样本信息

| 项目 | 值 |
|---|---|
| 路径 | `C:\Program Files (x86)\StreamDock\UpDateToolV3\default.img` |
| 大小 | 6,510,592 字节（0x635800） |
| MD5 | `af8624803c0be01c7c3519bd88bacb81` |
| SHA-256 | `3497cd42a84dc0181a28b224901c8801c6b6144ca6117f5642e24d65444be3a6` |
| 容器 | `AIC.FW` |
| platform | **d13x** |
| product | **D6-develop** |
| version | 1.0.0 |
| media | spi-nor |
| meta offset/size | 0x800 / 0x1000 |
| file offset/size | 0x1800 / 0x634000 |
| 尾部 | 0x635000–0x635800 为 0xFF 填充 |

---

## 2. 组件表（8 个）

| # | 组件名 | 分区 | 文件偏移 | 大小 | RAM | 属性 |
|---|---|---|---|---|---|---|
| 0 | image.updater.psram | — | 0x001800 | 0x7010（28 KB） | 0x30044000 | required;run |
| 1 | image.updater.spl | — | 0x009000 | 0x34210（213 KB） | 0x40100000 | required;run |
| 2 | image.info | — | 0x000000 | 0x800（2 KB） | — | required |
| 3 | image.target.spl | spl | 0x03D800 | 0x3B210（242 KB） | — | mtd;required |
| 4 | image.target.env | env;env_r | 0x079000 | 0x1000（4 KB） | — | mtd;optional |
| 5 | image.target.os | os | 0x07A000 | 0xA7800（686 KB） | — | mtd;required |
| 6 | **image.target.rodata** | **rodata** | **0x121800** | **0x14000（80 KB）** | — | mtd;optional |
| 7 | **image.target.data** | **data** | **0x135800** | **0x500000（5 MB）** | — | mtd;optional |

各组件的 CRC32 由 `upgcmdHid -i` 给出，例如 data 为 `0x5B7F48F0`、rodata 为 `0x5E9B8E53`。

---

## 3. 分区布局

从 SPL 内字符串（0x44750）读到：

```
mtd=spi0.0:512k(spl),128k(env),128k(env_r),2m(os),5m(rodata),5m(data);
```

| 分区 | 起始 | 大小 |
|---|---|---|
| spl | 0x000000 | 512 KB |
| env | 0x080000 | 128 KB |
| env_r | 0x0A0000 | 128 KB |
| os | 0x0C0000 | 2 MB |
| rodata | 0x2C0000 | **5 MB**（N4 Pro 是 3 MB） |
| data | 0x7C0000 | **5 MB**（N4 Pro 是 9 MB） |

> 与 N4 Pro 的 `3m(rodata),9m(data)` 明显不同，进一步证明这不是同一产品线。

---

## 4. 组件内容分析

### 4.1 spl（0x3D800，236 KB）

- 含 `bootcfg.txt`、`Image header`、`Pre-Boot Program`、`Load SPL image failed` 等引导字符串。
- 含分区表字符串与 `RO Data section` / `Data section` 解析逻辑。
- 无图像资源。

### 4.2 os（0x7A000，670 KB）

- RT-Thread + FAL 分区表：`FAL partition table`、`Partition read/write/erase error`。
- 含 `Image version: %s`、`data-offset`、`data-size`、`images` 等镜像/资源相关字符串。
- 无图像资源。

### 4.3 rodata（0x121800，80 KB）

实际有效内容很少，字节分布以 0x00 为主（32,046 / 81,920 为零）。发现 **3 张 JPEG**：

| 偏移 | 尺寸 | 说明 |
|---|---|---|
| 0x123600 | 800×480 | 启动/背景图 |
| 0x12374E | 160×96 | logo 小图 |
| 0x124816 | 160×96 | logo 小图 |

伴随字符串：`LOGO_J~1`、以及 Photoshop XMP 元数据（`from image/png to application/vnd.adobe.photoshop` 等），说明这些图由设计稿导出，不是点阵字库。

### 4.4 data（0x135800，5 MB）

- **5,178,218 / 5,242,880 字节为 0xFF（98.8%）**，实际几乎为空。
- 发现 2 张 160×96 JPEG（文件偏移 0x1E894E、0x1E9A1A）。
- 字符串：`logo_jpg_dir`、`test_logo.jpg`。
- 无字库、无图标网格、无协议表。

---

## 5. 是否为 N4 Pro 的 factory 区

**不是。** 依据：

| 对比项 | default.img | N4 Pro |
|---|---|---|
| platform | d13x | d21x |
| product | D6-develop | demo88_nor / N4 Pro |
| version | 1.0.0 | V4.N4 Pro_HJDF.02.015 |
| rodata | 5 MB 分区，实际 80 KB | 3 MB |
| data | 5 MB 分区，实际 5 MB 且 98.8% 为 0xFF | 9 MB |
| 资源 | 3+2 张 logo JPEG | 未知（不在 OTA 包内） |

它可以作为“带 rodata/data 的完整 AIC.FW”格式样本，但**平台与产品都不匹配，刷入 N4 Pro 会变砖**。

---

## 6. 结论与用途

1. `default.img` 是 UpDateToolV3 自带的 **D6 开发板演示固件**，不是 N4 Pro 的 factory 区；
2. 它证明了 AIC.FW 容器可以携带 `rodata`/`data` 组件，且官方工具链支持这两种分区；
3. 其资源区只有测试 logo，没有可复用的字库/图标；
4. **禁止刷入 N4 Pro**；仅供格式参考。

---

## 7. 附录：关键偏移

| 偏移 | 内容 |
|---|---|
| 0x000000 | AIC.FW 容器头 |
| 0x000800 | META 区（0x1000） |
| 0x001800 | image.updater.psram |
| 0x009000 | image.updater.spl |
| 0x03D800 | image.target.spl |
| 0x079000 | image.target.env |
| 0x07A000 | image.target.os |
| 0x121800 | image.target.rodata |
| 0x123600 | rodata 内 800×480 JPEG |
| 0x135800 | image.target.data |
| 0x1E894E | data 内 160×96 JPEG |
| 0x44750（spl 内） | 分区表字符串 |
| 0xFA064（os 内） | `FAL partition table` |
