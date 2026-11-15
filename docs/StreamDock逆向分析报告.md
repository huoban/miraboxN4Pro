# StreamDock 3.10.205.0918 逆向分析报告（N4 Pro / N4PRO_391A）

> 目标软件：`C:\Program Files (x86)\StreamDock\StreamDock.exe`
> 版本：3.10.205.0918（FileDescription: Stream Dock）
> 设备：N4 Pro，VID `0x5548` / PID `0x1008` / 序列号 `49490078391A`
> 固件：`V4.N4 Pro_HJDF.02.015`
> 分析方式：PE 私有 PDB 符号恢复 + 导出表/导入表 + 定点反汇编 + Frida 运行时 Hook + hidapi 原始报文采集
> 报告日期：2026-11-13

---

## 0. 结论摘要（先看这里）

1. **不存在 LetsGoClient 子进程**。实测子进程是 `CefViewWing.exe`（CEF 渲染）、`node20.exe`（每个插件一个）、`SplashScreen.exe`、`streamdockSwitchAudio.exe`。
2. **18618 不是“私有 TCP 控制协议”**，而是 **插件 WebSocket 服务端口**。每个插件以 `node20.exe ... index.js -port 18618 -pluginUUID <uuid> -registerEvent registerPlugin -info <json>` 启动并连回主进程。另存在 `SDTcpServer`（可配置端口的遥控协议）与 `SDWebsocketServer`，本次运行未对外监听 18618 以外的控制端口。
3. **N4 Pro 的 HID 通道是 hidapi + 厂商集合**：`usage_page=0xFFA0`、`usage=1`、`interface=0`；输出报告 1024 字节（实际写 1025 字节，含 ReportID 0x00），输入报告 512 字节，Feature 报告不支持（-1）。
4. **设备协议是 ASCII 命令头 + PNG 直传**：
   - 输出：`00 | "CRT" 00 00 <3字节命令> ...`，图像为 **原始 PNG 字节**按 1024 字节分片，末片补 0。
   - 输入：`"ACK" 00 00 "OK" 00 00 <索引> <状态>`；触摸坐标走 `"ACK" 00 "ARX" 00 00 00 <x:BE16> <y:BE16>`。
5. **按键编号**：输入事件索引 `1..10`（上排 1-5，下排 6-10，与用户定义一致）；**图像槽位**：触摸条 `1~4`、下排 `6~10`、上排 `11~15`，槽位 `5` 未使用（0.4.1 实机修正，早期“输入索引+5”仅适用于下排）。
6. **所有下发给设备的图像都旋转了 180°**（按键 112×112，触摸条 800×130）。
7. **配置加载顺序**：`StreamDockConfig.ini` → 当前设备/序列号 → `sdProfilePath` → 场景 `manifest.json`（`Pages.Current`）→ 子页面 `profiles\<page>\manifest.json` → 渲染并下发。`Pages.Current` 就是“退出时所在页面”。
8. **动作执行器**：内置动作由主程序原生执行（热键 SendInput、打开路径、多媒体、翻页、场景切换），插件类动作通过 18618 WebSocket 交给 node 插件；多动作/延时是独立动作 UUID，按顺序执行。
9. 官方软件存在明显内存/CPU 开销来源：CEF（QCefView + CefViewWing 多进程）、后台视频/GIF、插件 Node 进程。重制版可完全去掉。

---

## 1. 分析环境与证据文件

| 证据 | 路径 | 说明 |
|---|---|---|
| 函数符号表（67,414 个函数） | `docs/evidence/StreamDock_functions.tsv` | 由 `StreamDock.pdb` 恢复，含地址/大小/类名 |
| 类清单（593 个类） | `docs/evidence/StreamDock_class_list.tsv` | 模块划分依据 |
| 全量符号 | `docs/evidence/StreamDock_symbols.tsv` | 含 Data/PublicSymbol |
| SDLibrary1 导出表（1,405 项） | `docs/evidence/SDLibrary1_exports.txt` | 设备/HID/图像 API |
| 字符串表 | `docs/evidence/StreamDock_strings.txt`、`SDLibrary1_strings.txt` | 日志/命令/路径 |
| 活动场景配置快照 | `docs/evidence/manifest.N4Pro.active.json` | 当前“游戏”场景 |
| 配置 ini 快照 | `docs/evidence/StreamDockConfig.ini.snapshot` | 启动状态 |
| 启动全量 HID 写帧（1572 帧） | `docs/evidence/hid_frames_raw.txt` | 十六进制，1025B/帧 |
| 还原出的图像 | `docs/evidence/frames/*.png` | 触摸条 800×130、按键 112×112 |
| 实机输入帧 | `docs/evidence/input_frames.jsonl` | 按键/旋钮/触摸实测 |

> 注：`StreamDock.pdb`（92MB）与 exe 匹配，可恢复全部私有函数名；`SDLibrary1.pdb` 与当前 dll 版本不匹配（PDB 为 2026-09-01，dll 为 2026-09-23），因此 SDLibrary1 使用导出表 + 反汇编分析。

---

## 2. 进程与模块架构

### 2.1 进程树（实测）

```
StreamDock.exe (Qt5 + QCefView，主进程)
├─ CefViewWing.exe --type=gpu-process      CEF GPU
├─ CefViewWing.exe --type=utility           CEF 网络服务
├─ SplashScreen.exe                         启动闪屏
├─ node20.exe × N                           每个已安装插件一个进程
└─ streamdockSwitchAudio.exe                音频切换辅助
```

### 2.2 主程序依赖模块

| 模块 | 作用 |
|---|---|
| `Qt5Core/Gui/Widgets/Network/WebSockets` | UI、事件循环、WebSocket 服务 |
| `QCefView.dll` + `CefView/` | 内嵌浏览器（商店、插件属性面板、Web 插件） |
| `SDLibrary1.dll` | **设备层核心**：hidapi 封装、WinUSB 后端、图像编码、热键模拟、屏幕捕获、GIF、Zip、Keychain 等 |
| `libusb-1.0.dll` | USB 枚举/底层（hidapi 后端） |
| `ffmpeg-6.dll` | 视频背景/媒体解码（重制版不需要） |
| `libcurl.dll` | HTTP（商店/更新/上传） |
| `UpDateToolV3/`、`Watcher.exe` | 更新与守护 |
| `plugins\*.sdPlugin` | 内置/第三方动作插件（Node.js） |

### 2.3 关键类（来自 PDB 符号）

| 类 | 函数数 | 作用 |
|---|---|---|
| `SDProfileManager` | 3329 | 场景/页面/配置加载与保存 |
| `SDDeviceDisplayWidget` | 2312 | 设备渲染、页面切换、图像下发 |
| `SDPluginServer` | 1611 | 插件 WebSocket 服务（18618） |
| `SDPluginManager` | 882 | 插件进程生命周期 |
| `SDActionCanvasLabel` | 6206 | 单个按键/旋钮的渲染与动作执行 |
| `SDActionTouchBarWidget` | 342 | 触摸条渲染、拖动、分组 |
| `SDCoreConfiguration` | 1353 | 设备型号/能力/几何参数 |
| `DeviceInterfacesManage` | — | 设备接口枚举与串号管理 |
| `Utilities` | 3392 | JSON/配置/设备信息工具 |
| `SDTcpServer` / `SDWebsocketServer` | — | 遥控/外部控制服务（SDLibrary1 导出） |
| `DeviceCommand` | 16 | 设备命令（Mira75 校准/翻页/时间同步等） |
| `N4ProLightStripMenu` | — | N4 Pro 灯条设置 |

### 2.4 设备层 API（SDLibrary1 导出，节选）

```
SDDevice::openHidDevice(int vid, int pid, QString serial, int usagePage, int usage, SDConnectionMethod)
SDDevice::writeDataToHidDevice()          // 写线程
SDDevice::readDataFromHidDevice()         // 读线程
SDDevice::appendSendData(ImageStruct&, bool)
SDDevice::sendSettingsPackHead(int, uchar, uchar, DataType, bool)
SDDevice::addAdjustBrightnessPack(int, bool)
SDDevice::addScreenOffPack()
SDDevice::getClearAllCommand() / getClearCommand(uchar) / getFinishCommand()
SDDevice::getQUCMDCommand(u8,u8,u8,u8,u8)
SDDevice::sigReadData(QByteArray)
SDDevice::sigHandleKeyEvents(int, int)
SDDeviceWinUSB / SDWinUSB                  // WinUSB 后端（本机 N4Pro 未使用）
```

`SDDevice` 通过 Qt 信号把原始报文和按键事件抛给 `StreamDock.exe`；`SDDeviceWinUSB` 是另一套 Bulk 传输实现（其他型号使用）。

---

## 3. 启动与配置加载链（实测 + 符号验证）

### 3.1 时序（日志实测，2026-11-13 05:51 启动）

```
Init app - begin
Load preload html
MonitorApplicationsTool
SDWebsocketServer - end
SDPluginServer instance - end
SDProfileManager instance - end      ← 场景/页面已加载
init plugins manager
initWidget _ SDSetting / SDStore / SDToolbarWidget
SDDeviceDisplayWidget::onPageChanged ← 页面首次渲染
Init app - end
Connect to devices...                ← 设备枚举（5 个 HID 集合）
Open device - begin                  ← 打开厂商集合
hid_get_output_report_lenght 1024 / input 512 / feature -1
get firmware version "V4.N4 Pro_HJDF.02.015"
start hid read/write thread
SDDeviceDisplayWidget::refreshDisplay
```

**结论：先读配置并构建页面，再连接设备并刷屏。**

### 3.2 配置文件与读取顺序

**① 全局 ini**
`%APPDATA%\HotSpot\StreamDock\config\StreamDockConfig.ini`

| 段 | 键 | 实测值 | 用途 |
|---|---|---|---|
| `StreamDockCurrentDevice` | `Device` | `N4Pro` | 当前设备型号 |
| | `N4Pro\SerialNumber` | `49490078391A` | 当前设备序列号 |
| `N4Pro49490078391ADeviceConfig` | `sdProfileName` | `456AXNQI-...sdProfile` | 当前场景 |
| | `sdProfilePath` | `...\profiles\456AXNQI-...sdProfile` | 场景路径 |
| `DeviceBrightness` | `N4Pro49490078391A` | `60` | 亮度 0-100 |
| `RotationAngle` | `N4Pro49490078391A` | `0` | 用户旋转设置（非面板 180°） |
| `ScreenOffTime` | `N4Pro49490078391A` | `0` | 息屏时间（0=关） |
| `[N4Pro]` | `enableVibration` | `true` | 震动开关 |
| `[N4Pro49490078391A]` | `enableTouchBarMode` | `true` | 触摸条模式 |
| `[MBoxN4]` | `enableTouchClick` / `enableTouchSlide` | `true` / `true` | 点击/滑动 |
| `[MBoxN4+49490078391A+]` | `SlideSwitchMode` | `0` | 上下滑动切换模式 |
| `[N4Pro49490078391A]` | `VEDIO_*`、`constructImageData_*` | 见下 | 背景/视频合成几何 |

`constructImageData` 实测：`Top x=2,y=187,w=800,h=300`，`Bottom x=2,y=2,w=800,h=150`；视频裁剪/缩放同为 800×300 / 800×150。这是“背景层/视频层”的合成参数，重制版按需求忽略背景时可不用，但仍需走一次黑底刷新。

**② 场景档案**
`%APPDATA%\HotSpot\StreamDock\profiles\<sdProfileName>\manifest.json`

```json
{
  "Name": "游戏",
  "DeviceModel": "20GBA9901",
  "DeviceUUID": "N4Pro",
  "DeviceSerialNumber": "49490078391A",
  "Version": "1.0",
  "Pages": { "Current": "1JH7D037-...sdProfile", "Pages": [4 个页面] },
  "Actions": {
    "0,0": { ... }, "0,1": { ... }, "touchbar": { "Actions": [ ... ] }
  }
}
```

- `Pages.Current` = **退出时所在页面**（本次为 `1JH7D037...`，4 页中的第 2 页）。
- `Pages.Pages` 的第 3 项是场景自身（`456AXNQI...`），即“根页”。

**③ 子页面**
`<场景>\profiles\<page>\manifest.json`：只有 `Actions`（位置→动作）与 `Images\`。

### 3.3 位置编号规则（实测）

N4 Pro 使用 `"x,y"` 网格（5 列 × 3 行）：

| y | 含义 |
|---|---|
| `0` | 旋钮行：`0,0`~`3,0` = 旋钮 1~4（`Controller: Knob`） |
| `1` | **上排按键**：`0,1`~`4,1` = 按键 1~5 |
| `2` | **下排按键**：`0,2`~`4,2` = 按键 6~10 |

`touchbar` 是特殊键，值为 `{ "Actions": [ ... ] }`，即触摸条上的条目列表（Control Bar 图标）。

### 3.4 动作对象结构

```json
{
  "ActionID": "uuid",
  "UUID": "com.hotspot.streamdock.system.hotkey",
  "Name": "Hotkey",
  "Controller": "Keypad" | "Knob",
  "Settings": { ... },
  "State": 0,
  "States": [{
    "Image": "xxx.png",            // 相对当前 manifest 目录
    "Title": "奇迹出发",
    "ShowTitle": true,
    "TitleAlignment": "top"|"middle"|"bottom",
    "TitleColor": "#ffff00",
    "FontFamily": "HarmonyOS Sans",
    "FontSize": 20,
    "FontStyle": "Bold",
    "FontUnderline": false
  }]
}
```

---

## 4. 设备识别与 HID 通道

### 4.1 枚举结果（hidapi）

| 接口 | usage_page | usage | 用途 |
|---|---|---|---|
| MI_00 | `0xFFA0` (65440) | 1 | **厂商数据通道** |
| MI_01 Col01 | 1 | 6 | 键盘 |
| MI_01 Col02 | 12 | 1 | 消费者控制 |

官方日志中的枚举（5 个 HID 设备）：

```
vid 627 pid 1 usage 2 page 1
vid 5548 pid 1008 usage 1 page 65440   ← 目标
vid 5548 pid 1008 usage 6 page 1
vid 5548 pid 1008 usage 1 page 12
```

### 4.2 打开参数

```
openHidDevice(0x5548, 0x1008, "49490078391A", 0xFFA0, 1, SDConnectionMethod)
→ hid_get_output_report_lenght = 1024
→ hid_get_input_report_lenght  = 512
→ hid_get_feature_report_lenght = -1
```

设备字符串：Manufacturer `HOTSPOTEKUSB`，Product `HOTSPOTEKUSB HID DEMO`，Indexed `HOTSPOTEKUSB`。

### 4.3 读写线程

- 读线程：`hid_read_timeout(handle, buf, input_len+1, 3000ms)`，循环直到停止标志。
- 写线程：从队列取 `ImageStruct`，组包后 `hid_write`（1025 字节）。
- 停止/关闭：`hid_close` → `hid_exit`。

---

## 5. 输出协议（主机 → N4 Pro）

### 5.1 帧格式（实测，Frida Hook WriteFile）

```
每一帧 1025 字节：
[0]      ReportID = 0x00
[1..1024] 载荷（1024 字节）
```

命令载荷以 ASCII 头开始：

```
偏移:  0    1    2    3    4    5    6    7    8 ...
内容: 'C'  'R'  'T'  00   00   cmd0 cmd1 cmd2 ...
```

### 5.2 命令表（实测抓包 + 反汇编交叉验证）

| 命令 | 完整头（十六进制，载荷前 16 字节） | 参数 | 作用 |
|---|---|---|---|
| `DIS` | `43 52 54 00 00 44 49 53 00...` | 无 | 显示初始化/复位 |
| `LIG` | `43 52 54 00 00 4C 49 47 00 00 3C` | `[10]`=亮度 0-100 | 主亮度（实测 0x3C=60） |
| `LBLIG` | `43 52 54 00 00 4C 42 4C 49 47 3C` | `[10]`=亮度 | 标签/灯带亮度 |
| `QUCMD` | `43 52 54 00 00 51 55 43 4D 44 1F 11 00 11 00 11 00` | 5 字节查询参数 | 查询/握手 |
| `BGCLE` | `43 52 54 00 00 42 47 43 4C 45 02` / `03` | `[10]`=模式 2/3 | 背景清除（两层） |
| `BGPIC` | `43 52 54 00 00 42 47 50 49 43 00 01 3C A4 00 00 00 00 03 20 00 82 00 FF` | 见 5.3 | **触摸条图像** 800×130 |
| `BAT` | `43 52 54 00 00 42 41 54 00 00 62 7B 0B` | 见 5.4 | **按键图像** 112×112 |
| `CLE` | `43 52 54 00 00 43 4C 45 00 00 00 FF` | `[11]`=0xFF | 清空所有按键图 |
| `SETLB` | `43 52 54 00 00 53 45 54 4C 42 00` | 无 | 设置标签层 |
| `STP` | `43 52 54 00 00 53 54 50 00` | 反汇编另有 `[11]`=0x21/0x22/0x23 变体 | 传输结束/提交 |
| `CONNECT` | `43 52 54 00 00 43 4F 4E 4E 45 43 54` | 无 | 连接完成握手 |

### 5.3 BGPIC（触摸条图像）字段

以实测帧为例：

```
43 52 54 00 00 42 47 50 49 43 00 01 3C A4 00 00 00 00 03 20 00 82 00 FF
                              ^^ ^^^^^^^^^ ^^^^^^^^^^^ ^^^^^^^^^^^ ^^^^^
                             [10] [11..13]  [14..17]    [18..19]   [20..21] [22..23]
```

| 偏移 | 长度 | 含义 |
|---|---|---|
| 10 | 1 | 0x00（模式/固定） |
| 11..13 | 3 | **PNG 字节长度 24 位大端**（0x013CA4 = 81060） |
| 14..17 | 4 | 0 |
| 18..19 | 2 | 宽 800（BE16） |
| 20..21 | 2 | 高 130（BE16） |
| 22..23 | 2 | `00 FF` 结束标记 |

随后是 PNG 原始字节，按 **1024 字节/帧** 分片，末片补 0。总帧数 = `ceil(len/1024)`（实测 81060 → 80 帧）。

### 5.4 BAT（按键图像）字段

```
43 52 54 00 00 42 41 54 00 00 62 7B 0B
                           ^^^^^ ^^
                          [10..11] [12]
```

| 偏移 | 长度 | 含义 |
|---|---|---|
| 8..9 | 2 | 0 |
| 10..11 | 2 | **PNG 字节长度 BE16**（0x627B = 25211） |
| 12 | 1 | **按键图像索引 6..15** |

随后同样按 1024 字节分片传 PNG。

### 5.5 图像索引与物理按键映射（实测）

| 输入按键编号 | 物理位置 | 图像索引（BAT） |
|---|---|---|
| 1..5 | 上排左→右 | 6..10 |
| 6..10 | 下排左→右 | 11..15 |

实测 BAT 下发顺序：`11,12,13,14,15,6,7,8,9,10`（先下排后上排）。

### 5.6 整页刷新序列（实测）

```
DIS
LIG=60
QUCMD (1F 11 00 11 00 11 00)
BGCLE 02
BGCLE 03
BGPIC 800x130 PNG（触摸条，80 帧）
LIG=60
LBLIG=60
CLE 0xFF
BAT ×10（每键 14~27 帧）
SETLB
STP
（启动阶段会重复一次整页刷新）
CONNECT
```

### 5.7 图像方向

实测：**下发给设备的所有图像相对源图标旋转了 180°**。将抓到的按键图旋转 180° 后，文字与图标方向正常，且内容与配置中的标题一一对应。
→ 重制版在编码 PNG 前必须 `rotate(180°)`（按键与触摸条都适用）。

### 5.8 图像规格

| 对象 | 尺寸 | 格式 | 说明 |
|---|---|---|---|
| 按键 | 112×112 | PNG（RGBA） | 每键一张，含背景/图标/文字（官方合成后下发） |
| 触摸条 | 800×130 | PNG（RGBA） | 整条一张；Control Bar 模式按 160px × 5 分槽 |

---

## 6. 输入协议（N4 Pro → 主机）

输入报告固定 **512 字节**，`hid_read` 返回的字节 0 即载荷起始（无 ReportID 前缀）。

### 6.1 按键事件（实测）

```
41 43 4B 00 00 4F 4B 00 00 <idx> <state>
"A" "C" "K" 00 00 "O" "K" 00 00
```

- `idx`：1..10（按键 1..10）
- `state`：`1`=按下，`0`=松开

实测：
```
按键1 按下: 41 43 4B 00 00 4F 4B 00 00 01 01
按键1 松开: ... 01 00
按键6 按下: ... 06 01
按键6 松开: ... 06 00
```

### 6.2 旋钮（实测，旋钮 1）

| 动作 | 报文 | 说明 |
|---|---|---|
| 顺时针 1 格 | `41 43 4B 00 00 4F 4B 00 00 A1 00` | 每格一帧，索引 `0xA1`(161) |
| 按下 | `41 43 4B 00 00 4F 4B 00 00 37 01` | 索引 `0x37`(55)，状态 1 |
| 松开 | `... 37 00` | 状态 0 |

> 待验证：旋钮 2~4、逆时针方向、以及“旋钮索引是否按 A1..A4 / 37..3A 递增”。当前仅实测旋钮 1 顺时针与按下。

### 6.3 触摸条（实测）

**触摸按下事件：**

```
41 43 4B 00 00 4F 4B 00 00 00 01 ...
                          ^^ ^^
                         idx=0 state=1
```

**触摸坐标（连续上报）：**

```
41 43 4B 00 41 52 58 00 00 00 <x_hi> <x_lo> <y_hi> <y_lo>
"A" "C" "K" 00 "A" "R" "X" 00 00
```

| 偏移 | 含义 |
|---|---|
| 10..11 | X 坐标（BE16，0~800） |
| 12..13 | Y 坐标（BE16，0~130） |

实测水平滑动 X：`0x0076 → 0x0115`（118→277）连续变化；垂直滑动 Y：`0x0077 → 0x0006`（119→6）连续变化。
坐标方向与图像方向一致，**面板坐标是 180° 翻转的**（向右滑 X 增大）。

**触摸结束/手势事件（实测）：**

```
41 43 4B 00 00 4F 4B 00 00 B1 00
                          ^^ ^^
                        idx=0xB1(177) state=0
```

> 待验证：`0xB1` 是“触摸结束”还是“上下滑动切换模式”的业务码；`0xA1`/`0x37` 的完整索引空间需逐一实测。

### 6.4 主机侧确认报文

设备对 `BGPIC/BAT/CLE` 等命令会回：

```
41 43 4B 00 00 4F 4B 00 00 01 00 ...
```

即 `ACK 00 00 OK 00 00 <cmd/idx> <state>`，与按键事件同格式，需按 `idx` 区分（实测 idx=0x00/0x01 为命令确认）。

---

## 7. 渲染与下发管线（官方实现 → 重制映射）

### 7.1 官方层级

1. **背景层**：场景 `Images\` 中的背景图 / 视频（GIF/MP4），按 `constructImageData_*` 与 `VEDIO_*` 合成到 800×300（上区）与 800×150（下区）。
2. **图标层**：`States[].Image`，按动作状态（State）选择。
3. **文字层**：`Title`、`FontFamily`、`FontSize`、`FontStyle`、`TitleColor`、`TitleAlignment`、`ShowTitle`、`FontUnderline`。
4. **触摸条**：`Actions["touchbar"].Actions[]` 列表，Control Bar 模式横向排布（160px/项，可滑动）；旋钮模式显示 4 个旋钮对应内容。
5. 合成结果 → PNG 编码（`DataFormatConversion::imageToBin` / `QImageToQByteArray`）→ 旋转 180° → 分片 → 设备。

### 7.2 重制版要求（对齐用户需求）

- 忽略背景层与视频/GIF：按键与触摸条以**纯黑底**合成图标+文字后下发。
- 切换页面时仍完整走一次刷新流程（黑底），但不做连续动画刷新。
- 不处理旋钮 RGB 灯效。
- 按键 112×112、触摸条 800×130，PNG RGBA，旋转 180°。
- 触摸条 Control Bar：5 个条目 ×160px 一屏；条目多于 5 个时按官方规则横向滚动（实测配置为 6 项，显示前 5 项）。
- 旋钮模式：触摸条显示 4 个旋钮对应内容（官方会下发不同的 800×130 图，本次未抓取切换后的图，待实现阶段补抓）。

---

## 8. 动作执行器

### 8.1 动作分类与实测参数

本次 N4 Pro 配置中实际使用的动作：

| UUID | 名称 | Settings 关键字段 | 执行方式 |
|---|---|---|---|
| `com.hotspot.streamdock.system.hotkey` | Hotkey | `Hotkeys[]`: `VKeyCode`, `KeyModifiers`(65536=Ctrl), `KeyShift/KeyCtrl/KeyAlt`, `Coalesce` | Win32 `SendInput`（SDLibrary1 `SimulatedKeyboardEvent`） |
| `com.hotspot.streamdock.system.open` | Open | `path`（可执行文件/文件/文件夹）, `openShowState` | `ShellExecute` / QDesktopServices |
| `com.hotspot.streamdock.system.website` | Website | `path`, `browser`, `openInBrowser` | 默认浏览器或内嵌 CEF |
| `com.hotspot.streamdock.system.multimedia` | Multimedia | `actionIdx` | 系统多媒体键（播放/暂停/下一首等） |
| `com.hotspot.streamdock.system.volume` | SystemVolume | `actionIdx` | 音量键/系统音量 |
| `com.hotspot.streamdock.page.change` | change page | 无 | 内部翻页（`SDPages`） |
| `com.hotspot.streamdock.profile.rotate` | Scene Shift | `DeviceUUID`, `ProfileUUID` | 切换场景 |
| `com.hotspot.streamdock.multiactions.routine` | Multi Action | 子动作列表（顺序执行） | 逐个 `performActionKeyDown` |
| `com.hotspot.streamdock.multiactions.delay` | Delay | 延时毫秒 | 定时器等待，串行 |
| `com.hotspot.streamdock.multiactions.toggle` | Cycle | 多状态轮换 | 状态机 |
| `com.hotspot.streamdock.multiactions.LunBo` | Action Carousel | 轮播 | 状态机 |
| `com.hotspot.streamdock.multiactions.ActionWheel/ActionTrigger` | 旋钮多动作 | 旋钮分段 | 旋钮事件分派 |
| `com.hotspot.streamdock.network.udp` | UDP | 地址/端口/报文 | UDP 发送（OSC 可由此承载） |

> 未发现独立的 `http request` 或 `osc` 内置插件；HTTP 由 `SDHttpRequest`/libcurl 在程序内部使用（商店/更新），OSC 需通过 UDP 动作或第三方插件。重制版应内置：HTTP、UDP/OSC、程序/脚本、热键、延时、串行/并行多动作。

### 8.2 执行时序（反汇编 + 日志）

- 按键按下：`SDActionCanvasLabel::onKeyDown` → `performActionKeyDown` → 动作分派 → 状态图切换 → `sigSendImageData` → 设备写帧。
- 按键松开：`performActionKeyUp`（部分动作仅在松开触发）。
- 旋钮：`dialRotate` / `dialDown` / `dialUp`，支持 `KnobOperatingGroup`（分段动作）。
- 多动作：`SDMultiActionListWidget` 配置，顺序执行子动作；`Delay` 作为子动作插入。
- 触摸条：`SDActionTouchBarWidget::performActionKeyDownItem` / `KeyUpItem`。

### 8.3 重制版动作执行器设计（建议）

```
ActionContext { device, page, position, isKeyDown }
dispatch(action):
  switch action.UUID:
    hotkey      -> SendInput(按 Settings.Hotkeys 顺序按下/抬起)
    open        -> ShellExecute(path)
    website     -> ShellExecute(url) 或内置浏览器
    multimedia  -> keybd_event(VK_MEDIA_*)
    volume      -> 系统音量 API
    page.change -> 切换页并刷新
    profile.rotate -> 切换场景并刷新
    multiactions.routine -> 串行执行子动作（可并行分支）
    multiactions.delay   -> sleep(ms) 后继续
    network.udp -> UdpSocket.send_to
    http        -> reqwest（新增，按用户要求）
    osc         -> rosc（新增，按用户要求）
```

按键“按下/松开”语义、`Coalesce`（连按合并）、旋钮方向/步进都要按配置原样实现。

---

## 9. 插件体系与 18618 端口

### 9.1 实测

- 主进程监听 `::18618`（WebSocket，插件专用）。
- 每个插件进程命令行：

```
"C:\Program Files (x86)\StreamDock\node\node20.exe" <plugin>\plugin\index.js \
  -port 18618 -pluginUUID <32位UUID> -registerEvent registerPlugin \
  -info "{\"application\":{...},\"devices\":[{\"id\":\"...\",\"name\":\"N4Pro\",\"size\":{\"columns\":5,\"rows\":3},\"type\":0}],...}"
```

- 插件注册后主进程日志：`SDPluginServer::onTextMessageReceived "Plugin ... is now connected"`。
- 事件：`keyDown/keyUp`、`dialDown/dialRotate/dialUp`、`willAppear/willDisappear`、`setImage/setTitle/setState/setSettings`、`propertyInspectorDidAppear` 等（`SDPluginServer` 导出信号）。
- 其他端口：`23519`（CEF DevTools，127.0.0.1）、`24508`/`28619`（内置 MCP 服务）、`56018`（商店 WebSocket）。

### 9.2 SDTcpServer（非 18618）

`SDLibrary1` 导出 `SDTcpServer::startListen(port)`，信号为：

```
sigClientConnected(QTcpSocket*, QString, QString, QString)
sigClientKeyDown(QTcpSocket*, int, int)
sigClientKeyUp(QTcpSocket*, int, int)
sigClientDialPressed(QTcpSocket*, int, int)
sigClientDialRotate(QTcpSocket*, int, int, bool)
```

这是给手机/遥控端用的控制协议，端口由调用方传入（本次运行未监听）。重制版不需要。

### 9.3 对重制版的含义

- 若只实现“原生动作”，可以不启动任何端口、不加载 Node 插件。
- 若要兼容插件动作（OBS、Spotify、天气、频谱等），需要实现 18618 WebSocket 服务端 + 插件进程管理；建议第一阶段不做，按配置把插件动作降级为“提示/忽略”。

---

## 10. manifest 数据结构（完整字段）

```jsonc
{
  "Name": "游戏",
  "DeviceModel": "20GBA9901",       // N4Pro 型号码
  "DeviceUUID": "N4Pro",
  "DeviceSerialNumber": "49490078391A",
  "Version": "1.0",
  "Pages": {
    "Current": "<page-uuid>.sdProfile",
    "Pages": ["<page-uuid>.sdProfile", ...]   // 可包含场景自身
  },
  "Actions": {
    "x,y": {                         // x=0..4, y=0..2；y=0 为旋钮
      "ActionID": "...",
      "UUID": "com.hotspot....",
      "Name": "...",
      "Controller": "Keypad"|"Knob",
      "Settings": { ... },
      "State": 0,
      "States": [ { "Image":"a.png", "Title":"...", "ShowTitle":true,
                    "TitleAlignment":"top", "TitleColor":"#ffff00",
                    "FontFamily":"HarmonyOS Sans", "FontSize":20,
                    "FontStyle":"Bold", "FontUnderline":false } ],
      "SoftwareSettings": {}
    },
    "touchbar": { "Actions": [ <同上动作对象> ] }
  }
}
```

图片路径相对于 `manifest.json` 所在目录（`Images\` 或子页面目录）。

---

## 11. 重制软件（Rust）落地要点

> 以下是分析结论到实现的映射，具体代码方案将另行提交《代码任务执行拆解方案》并经确认后实施。

1. **单 exe、无 UI**：双击=控制台，`-serve`=静默后台；不引入 CEF/Qt/Node。
2. **只读配置**：读取 ini + manifest，不写回；场景/页面/动作/图标全部从 `%APPDATA%\HotSpot\StreamDock` 读取。
3. **HID 唯一通道**：`hidapi`（Windows）打开 `VID 0x5548 / PID 0x1008 / usage_page 0xFFA0 / usage 1`，输出 1025 字节帧、输入 512 字节。
4. **启动刷屏**：读取 `Pages.Current` → 渲染当前页 → 发送 `DIS/LIG/QUCMD/BGCLE/BGPIC/CLE/BAT×10/SETLB/STP`。
5. **图像**：按键 112×112、触摸条 800×130，RGBA PNG，**旋转 180°**，1024 字节分片。
6. **输入**：解析 `ACK 00 00 OK`、`ACK 00 ARX`，按键 1..10、旋钮、触摸坐标、手势。
7. **动作**：热键（SendInput）、打开、网址、多媒体、音量、翻页、场景切换、多动作/延时、HTTP、UDP/OSC。
8. **降耗**：无背景视频、无 RGB、无插件 Node 进程；仅在页面切换/图标变化时刷屏。
9. **模式**：Control Bar 与旋钮模式的触摸条图需各缓存一张，切换时下发对应 800×130 图。

---

## 12. 未验证项与风险

| # | 项目 | 现状 | 影响 |
|---|---|---|---|
| 1 | 旋钮 2~4 的事件索引 | 仅实测旋钮 1（A1/37） | 需补测或按 A1..A4/37..3A 推断 |
| 2 | 逆时针索引/方向位 | 未测 | 旋钮方向可能反 |
| 3 | 上下滑动切换模式后的触摸条图 | 未抓 | 旋钮模式条图需补抓 |
| 4 | `SETLB` 参数 | 实测全 0 | 可能是“标签层提交”，对纯图标方案可省略 |
| 5 | `STP` 的 0x21/0x22/0x23 变体 | 反汇编发现，实测为 0 | 不同型号/固件分支 |
| 6 | `BGCLE` 02/03 的层级语义 | 推断为两层背景 | 黑底方案可固定发 02+03 |
| 7 | 插件动作兼容 | 未实现 | 第一阶段可忽略 |
| 8 | 震动指令 | 配置有开关，本次未抓到 | 需要时补抓按键触发时的输出帧 |
| 9 | 息屏 | `ScreenOffTime=0`，未触发 | 可按需实现 `addScreenOffPack` |

---

## 附录 A：SDLibrary1 关键导出（协议相关）

```
SDDevice::openHidDevice / closeHidDevice / isHidDeviceOpen
SDDevice::appendSendData(ImageStruct&, bool)
SDDevice::sendSettingsPackHead(int, uchar, uchar, DataType, bool)
SDDevice::addAdjustBrightnessPack(int, bool) / sendAdjustBrightnessPack
SDDevice::addScreenOffPack / sendScreenOffPack
SDDevice::getClearAllCommand / getClearCommand(uchar) / getFinishCommand
SDDevice::getQUCMDCommand(u8,u8,u8,u8,u8)
SDDevice::addKeyboardDownCommand / addKeyboardUpCommand
SDDevice::sigReadData(QByteArray) / sigHandleKeyEvents(int,int)
DataFormatConversion::imageToBin / gifToBin / QImageToQByteArray
SDTcpServer::startListen(port) / sigClientKeyDown / sigClientDialRotate
SDWinUSB::findDevice / getAllConnectedDevices（WinUSB 后端，N4Pro 未用）
```

## 附录 B：实测报文速查

```
# 亮度 60
00 43 52 54 00 00 4C 49 47 00 00 3C 00...
# 清空按键
00 43 52 54 00 00 43 4C 45 00 00 00 FF 00...
# 触摸条 800x130，PNG 长度 0x013CA4
00 43 52 54 00 00 42 47 50 49 43 00 01 3C A4 00 00 00 00 03 20 00 82 00 FF ...
# 按键 idx=11，PNG 长度 0x627B
00 43 52 54 00 00 42 41 54 00 00 62 7B 0B ...
# 按键1 按下 / 松开
41 43 4B 00 00 4F 4B 00 00 01 01 / ... 01 00
# 触摸坐标帧
41 43 4B 00 41 52 58 00 00 00 01 15 00 3F
# 旋钮1 顺时针一格 / 按下 / 松开
41 43 4B 00 00 4F 4B 00 00 A1 00 / ... 37 01 / ... 37 00
```

---

*本报告基于 2026-11-13 的本机实测与静态分析，结论区分“实测”“反汇编推断”“待验证”。所有原始证据保存在 `docs/evidence/`，可复核。*
