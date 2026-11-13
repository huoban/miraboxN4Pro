# AI 开发功能说明（N4ProCore）

> 本文件为项目主文档，记录目录结构、功能清单、类定义、开发进度与版本迭代记录。
> 关联文档：[StreamDock 逆向分析报告](docs/StreamDock逆向分析报告.md)（协议基线）

---

## 3.1 项目目录结构与功能说明

```
d:\Ai\N4PRO
├── n4pro-core\                     【核心功能】Rust 重制软件工程 【关联模块】全部 【修改权限】可新增/修改源码，禁止删除已上线模块
│   ├── Cargo.toml                  【核心功能】依赖与 release 优化配置 【关联模块】编译流程 【修改权限】可调整依赖
│   ├── build.bat                   【核心功能】一键 release 编译入口 【关联模块】编译流程 【修改权限】可维护
│   ├── src\
│   │   ├── main.rs                 【核心功能】参数解析、单实例、控制台/静默切换 【关联模块】app 【修改权限】可修改业务，禁止改动入口框架结构
│   │   ├── app.rs                  【核心功能】页面状态机、刷新、输入分派、动作效果 【关联模块】config/device/render/actions 【修改权限】可修改
│   │   ├── logging.rs              【核心功能】运行目录日志与 2MB×3 滚动 【关联模块】全模块 【修改权限】可修改
│   │   ├── config\                 【核心功能】ini 与 manifest 解析（只读） 【关联模块】app 【修改权限】可新增字段，禁止写回配置
│   │   ├── device\                 【核心功能】HID 枚举/读写、协议帧、输入解析 【关联模块】app 【修改权限】协议层已实测，改动需回归测试
│   │   ├── render\                 【核心功能】112×112 按键、800×130 触摸条、GDI 文字、PNG 【关联模块】app 【修改权限】可优化视觉效果
│   │   ├── actions\                【核心功能】动作执行器（热键/打开/媒体/网络/多动作） 【关联模块】app 【修改权限】可扩展动作类型
│   │   └── platform\win.rs         【核心功能】Win32 FFI（HID/GDI/Shell/控制台） 【关联模块】全模块 【修改权限】仅在新增系统调用时更新
│   ├── logs\                       【核心功能】运行日志目录 【关联模块】logging 【修改权限】禁止手动修改，程序自动生成
│   └── target\                     【核心功能】编译输出 【关联模块】编译流程 【修改权限】禁止手动修改
├── docs\
│   ├── StreamDock逆向分析报告.md    【核心功能】协议与模块逆向基线 【关联模块】全项目 【修改权限】仅可追加修订
│   └── evidence\                   【核心功能】原始证据（符号/帧/图像/配置快照） 【关联模块】逆向分析 【修改权限】只读
└── AI开发功能说明.md               【核心功能】本文档 【关联模块】全项目 【修改权限】仅追加/更新对应模块
```

---

## 3.2 项目全量功能清单与描述

| 序号 | 功能名称 | 功能详细描述（含输入输出） | 适用场景 | 依赖模块 | 当前状态 | 上线版本 |
|---|---|---|---|---|---|---|
| 1 | 控制台/静默启动 | 默认保留控制台；`-serve` 调用 FreeConsole 脱离控制台，仅写日志 | 日常启动/后台常驻 | platform | 已上线 | 0.1.0 |
| 2 | 只读配置加载 | 输入 ini 路径，输出设备、序列号、场景路径、亮度、触摸模式；不写回 | 启动 | config | 已上线 | 0.1.0 |
| 3 | 场景/页面解析 | 输入 manifest.json，输出 Actions、Pages、touchbar、图片路径 | 启动/切页/切场景 | config | 已上线 | 0.1.0 |
| 4 | HID 设备打开 | 按 VID/PID/usage 枚举并打开厂商集合，输出 1025B 帧、输入 512B | 启动 | platform/device | 已上线 | 0.1.0 |
| 5 | HID 帧协议 | 构造 DIS/LIG/LBLIG/QUCMD/BGCLE/BGPIC/BAT/CLE/SETLB/STP/CONNECT | 刷屏/握手 | device | 已上线 | 0.1.0 |
| 6 | 按键渲染 | 112×112 黑底+图标+文字，旋转 180°，PNG 编码 | 整页刷新/按键状态 | render | 已上线 | 0.1.0 |
| 7 | 触摸条渲染 | 800×130，Control Bar 5×160px（可滚动）/旋钮模式 4×200px | 刷屏/滑动手势 | render | 已上线 | 0.1.0 |
| 8 | 整页刷新 | 输出 BGCLE→BGPIC→CLE→BAT×10→SETLB→STP 帧序列 | 启动/切页 | app | 已上线 | 0.1.0 |
| 9 | 输入事件解析 | 解析按键 1~10、旋钮旋转/按下、触摸坐标、手势 | 运行期 | device | 已上线 | 0.1.0 |
| 10 | 页面切换 | 触摸条左右滑动切页并重刷 | 运行期 | app | 已上线 | 0.1.0 |
| 11 | 模式切换 | 触摸条上下滑动切换 Control Bar/旋钮模式并换图 | 运行期 | app | 已上线 | 0.1.0 |
| 12 | 热键动作 | 解析 Hotkeys[]，SendInput/键盘事件模拟 | 按键动作 | actions | 已上线 | 0.1.0 |
| 13 | 打开动作 | 打开程序/文件/文件夹/网址 | 按键动作 | actions | 已上线 | 0.1.0 |
| 14 | 多媒体/音量 | 系统媒体键与音量键 | 旋钮/按键动作 | actions | 已上线 | 0.1.0 |
| 15 | HTTP 动作 | GET/POST，3 秒超时 | 按键动作 | actions | 已上线 | 0.1.0 |
| 16 | UDP/OSC 动作 | UDP 发送、OSC 1.0 编码发送 | 按键动作 | actions | 已上线 | 0.1.0 |
| 17 | 多动作/延时 | 串行/并行子动作、延时等待 | 复杂动作 | actions | 已上线 | 0.1.0 |
| 18 | 场景切换 | 按 ProfileUUID 切换场景并重载 | 旋钮/按键动作 | app | 已上线 | 0.1.0 |
| 19 | 插件动作跳过 | 第三方插件 UUID 记录日志后跳过 | 兼容官方配置 | actions | 已上线 | 0.1.0 |
| 20 | 帧转储/试运行 | `--dump-frames` 导出实际帧；`--dry-run` 只记录不执行 | 测试/回归 | app | 已上线 | 0.1.0 |
| 21 | 旋钮模式条图 | 使用根配置 0,0~3,0 渲染 4 分区 | 旋钮模式 | render | 待优化 | 0.1.0 |
| 22 | 旋钮 2~4/逆时针 | 事件索引表已预留，需补测 | 旋钮 | device | 待优化 | 0.1.0 |
| 23 | 息屏/震动 | 协议已分析，尚未接入动作 | 可选 | device | 未开发 | - |

---

## 3.3 全量类定义说明

### 类名称：App
- 所属文件：`n4pro-core/src/app.rs`
- 核心功能：应用核心状态机，负责页面装载、整页刷新、输入分派、动作效果应用与日志。
- 核心属性：
  - `cfg: AppConfig` 运行配置
  - `scene: Scene` 当前场景
  - `page_dir: PathBuf` 当前页图片基准目录
  - `page_idx: usize` 当前页索引
  - `actions: BTreeMap<String, ActionDef>` 当前页动作
  - `cache: ImageCache` PNG 缓存
  - `knob_mode: bool` 触摸条模式
  - `scroll: i32` Control Bar 滚动偏移
  - `tx: Option<Sender<Vec<u8>>>` HID 写队列
  - `dry_run/no_device/once: bool` 测试开关
- 核心方法：
  - `new(cfg, dry_run, dump_frames, no_device, once) -> Result<App, String>`：加载场景并定位页面
  - `run(&mut self)`：打开设备、发送初始化帧、进入事件循环
  - `refresh_all(&mut self)`：整页刷新
  - `handle_report(&mut self, &[u8])`：解析并处理输入
  - `switch_page(&mut self, i32)` / `switch_scene(&mut self, &str)`
- 标准用法示例：
  ```rust
  let cfg = config::load(None)?;
  let mut app = App::new(cfg, true, false, false, true)?;
  app.run();
  ```
- 依赖关系：config、device、render、actions、platform
- 迭代记录：0.1.0 新建

### 类名称：HidDevice
- 所属文件：`n4pro-core/src/device/hid.rs`
- 核心功能：枚举并打开 N4 Pro 厂商 HID 集合，提供 1025B 写、512B 读。
- 核心属性：`handle: HANDLE`、`output_len: usize`、`input_len: usize`
- 核心方法：`open_n4pro() -> Result<HidDevice, String>`、`write_frame(&[u8])`、`read_report(&mut [u8])`
- 标准用法示例：
  ```rust
  let dev = HidDevice::open_n4pro()?;
  dev.write_frame(&protocol::lig(60))?;
  ```
- 依赖关系：platform::win（SetupAPI/Hid/kernel32）
- 迭代记录：0.1.0 新建

### 类名称：Canvas
- 所属文件：`n4pro-core/src/render/canvas.rs`
- 核心功能：RGBA 黑底画布，图像 cover/contain 合成、180° 旋转、PNG 编码。
- 核心属性：`width: u32`、`height: u32`、`data: Vec<u8>`
- 核心方法：`new_black`、`draw_cover`、`draw_contain`、`rotate180`、`encode_png`
- 标准用法示例：
  ```rust
  let mut c = Canvas::new_black(112, 112);
  c.draw_cover(&img, 0, 0, 112, 112);
  c.rotate180();
  let png = c.encode_png()?;
  ```
- 依赖关系：image
- 迭代记录：0.1.0 新建

### 类名称：ActionDef / StateDef
- 所属文件：`n4pro-core/src/config/manifest.rs`
- 核心功能：描述一个动作及其多状态外观（图片、标题、字体、颜色、对齐）。
- 核心属性：`uuid`、`name`、`controller`、`settings: Value`、`states: Vec<StateDef>`、`state_index`
- 核心方法：`state() -> StateDef`、`parse_action(&Value)`、`parse_state(&Value)`
- 标准用法示例：
  ```rust
  let def = parse_action(&json);
  let st = def.state();
  ```
- 依赖关系：serde_json
- 迭代记录：0.1.0 新建

### 类名称：Scene
- 所属文件：`n4pro-core/src/config/manifest.rs`
- 核心功能：场景根档案（Pages.Current/Pages、根动作、触摸条条目）。
- 核心属性：`dir`、`name`、`current_page`、`pages`、`root_actions`、`touchbar`
- 核心方法：`load_scene(&Path)`、`load_page_actions(&Path)`
- 标准用法示例：
  ```rust
  let scene = load_scene(profile_dir)?;
  ```
- 依赖关系：serde_json
- 迭代记录：0.1.0 新建

### 类名称：ImageCache
- 所属文件：`n4pro-core/src/render/cache.rs`
- 核心功能：按签名缓存 PNG，避免重复渲染。
- 核心属性：`map: HashMap<String, Vec<u8>>`
- 核心方法：`get`、`put`、`clear`
- 标准用法示例：`cache.put(sig, png);`
- 依赖关系：std
- 迭代记录：0.1.0 新建

### 枚举：InputEvent
- 所属文件：`n4pro-core/src/device/input.rs`
- 核心功能：输入事件模型（按键/触摸/旋钮/手势）。
- 变体：`Key{index,down}`、`TouchDown`、`TouchMove{x,y}`、`TouchUp`、`GestureEnd`、`DialRotate{id}`、`DialPress{id,down}`、`Unknown{index,state}`
- 核心方法：`parse(&[u8]) -> Option<InputEvent>`
- 标准用法示例：`if let Some(ev) = parse(&buf) { ... }`
- 依赖关系：无
- 迭代记录：0.1.0 新建

### 枚举：Effect
- 所属文件：`n4pro-core/src/actions/mod.rs`
- 核心功能：动作执行后需要主程序处理的效果。
- 变体：`None`、`NextPage`、`PrevPage`、`SwitchScene(String)`
- 核心方法：`actions::execute(&ActionDef, dry_run, depth) -> Effect`
- 标准用法示例：`let effect = execute(&def, false, 0);`
- 依赖关系：config
- 迭代记录：0.1.0 新建

---

## 3.4 项目进度与后续开发规划

### 当前开发进度
- 已完成里程碑：
  1. 逆向分析完成（协议/配置/进程/动作），报告与证据落盘。
  2. Rust 工程 0.2.0：配置、HID（重叠 I/O）、协议、渲染、输入、动作、心跳、静默模式、日志、单元测试。
  3. 离线验证：整页帧结构正确，按键/触摸条预览与官方截图一致。
  4. 实机验证：10 键图像与文字正常显示；触摸条 4 区显示；按键/旋钮/触摸上报；上下滑模式切换；写入不再阻塞。
- 当前所处开发阶段：实机联调收尾（0.2.0 已通过按键与输入验收）。
- 实机验收结论（2026-11-13）：
  1. 10 个物理按键显示正常。
  2. 按键、旋钮、触摸点击/滑动全部有响应。
  3. 触摸条部分 UI 显示错误，待原图/对比图后修正。
- 待解决 BUG/问题：
  1. 触摸条 UI 布局/内容与官方不一致（阻塞于原图与对比图）。
  2. 左右滑动翻页后触摸条未刷新（BGPIC 协议未接入）。
  3. Control Bar 快捷点击码 0x40~0x43 未映射到子动作。
  4. 长时间连续操作后设备偶发掉线（USB 侧再次枚举不到，需排查线材/供电/固件心跳）。
- 阻塞项：触摸条 UI 修正需用户提供原图与对比图；设备掉线需硬件侧复测。

### 后续开发规划
| 阶段 | 阶段目标 | 交付物 | 时间节点 | 优先级 | 负责人 |
|---|---|---|---|---|---|
| 联调 | 实机验证 S1~S8 | 验收记录 | 2026-11-13 | 高 | AI |
| 补测 | 旋钮全索引/旋钮模式条图 | 索引表更新 | 待定 | 中 | AI+用户 |
| 优化 | 内存/CPU 采样与图像差异校准 | 性能记录 | 待定 | 中 | AI |
| 扩展 | 息屏/震动/亮度动作 | 新动作 | 待定 | 低 | AI |

---

## 版本迭代记录

### 版本号：0.1.0
- 编译时间：2026-11-13 06:52
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（PowerShell 调用 `cargo build --release`，Rust 1.95.0 x86_64-pc-windows-msvc）
- 编译结果：编译成功；单元测试 8/8 通过
- 本次版本变更概要：
  1. 新增：Rust 工程全部模块（配置/HID/协议/渲染/输入/动作/日志/静默模式）、`build.bat`、本说明文档
  2. 修改：无（首个版本）
  3. 删除：无
  4. 兼容说明：只读官方配置，不修改任何官方文件；不可与官方软件同时写设备

---

### 版本号：0.1.1
- 编译时间：2026-11-13 07:01
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-pc-windows-msvc）
- 编译结果：编译成功；单元测试 8/8 通过（0.1.0 基线）
- 本次版本变更概要：
  1. 新增：`--probe`（仅 DIS+LIG 安全探测）、`--no-device`、`--once` 测试参数
  2. 修改：HID 读缓冲改为设备上报的 513 字节，修复错误码 1784；写入增加 2ms/帧、STP/CONNECT 30ms 节流；触摸条文字按 160px 分区绘制；图标支持 `Images\` 子目录；序列号读取优先精确键并过滤空值；触摸条图标改为 cover 填充
  3. 删除：无
  4. 兼容说明：协议帧与官方实测一致；不影响官方配置

---

### 版本号：0.2.0
- 编译时间：2026-11-13 08:34
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 8/8 通过
- 本次版本变更概要：
  1. 新增：2 秒 RGBS 心跳；重叠 I/O 单句柄；触摸条槽位 1~4 渲染（176×112）；`--slot-test` 槽位标定；`--log-input`；固件模式事件 0xB1/0xB2
  2. 修改：输入解析跳过 1 字节 ReportID；按键槽位映射按实测调整（slot = key + 5）；图片间 60ms、分片 5ms 节流；移除 BGPIC/BGCLE（会覆盖按键层）；崩溃信息写入日志
  3. 删除：独立读句柄方案（改为重叠 I/O）
  4. 兼容说明：只读配置；实机已验证 10 键显示、触摸条 4 区、按键/旋钮/触摸上报、上下滑模式切换

---

### 版本号：0.3.0
- 编译时间：2026-11-13 09:01
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 8/8 通过
- 本次版本变更概要：
  1. 新增：旋钮模式内置图标回退（场景切换/多媒体/音量/翻页页码）；Control Bar 800×130 整条渲染（5 按钮 + 箭头 + 滚动吸附）；`--mode knob|bar`、`--no-bgpic`
  2. 修改：启动模式改读 `[N4Pro<serial>] enableTouchBarMode`（true=Control Bar）；模式切换时旋钮模式先发黑条再发 1~4 区；翻页后重新绑定旋钮动作与页码；多媒体默认标题避免重复
  3. 删除：无
  4. 兼容说明：6~15 按键渲染/映射未改动；配置只读；Control Bar 仅在对应模式发送 BGPIC

---

### 版本号：0.3.1
- 编译时间：2026-11-13 09:16
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 8/8 通过
- 本次版本变更概要：
  1. 新增：Control Bar 按钮叠加 `States[].Title` 文字层（字体/字号/颜色/对齐）
  2. 修改：Control Bar 整条图增加 180° 预旋转，修正真机文字倒置与左右顺序镜像
  3. 删除：无
  4. 兼容说明：仅影响触摸条 800×130 图；按键 6~15 不变

---

### 版本号：0.3.2
- 编译时间：2026-11-13 09:28
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 8/8 通过
- 本次版本变更概要：
  1. 新增：无
  2. 修改：文字字号按 Qt pt → GDI 像素换算（96/72，20pt≈27px），解决整体偏小；按键标题内边距 4→1、旋钮区 4→2，避免省略号截断
  3. 删除：无
  4. 兼容说明：仅影响文字渲染；按键/触摸条布局与协议不变

---

### 版本号：0.3.3
- 编译时间：2026-11-13 09:34
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：无
  2. 修改：字号换算由 pt×96/72 回调一档 10% 至 pt×1.2（20pt≈24px），解决整体偏大
  3. 删除：无
  4. 兼容说明：仅文字大小变化

---

## 最新版本对比变更概要

### 对比版本：0.3.3 VS 0.3.2
- 对比生成时间：2026-11-13 09:35
- 新增内容：无
- 修改内容：字号缩小一档（10%）
- 删除内容：无
- 核心影响说明：按键与触摸条文字比例进一步贴近官方
