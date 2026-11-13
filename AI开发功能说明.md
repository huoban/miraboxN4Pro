# AI 开发功能说明（N4ProCore）

> 本文件为项目主文档，记录目录结构、功能清单、类定义、开发进度与版本迭代记录。
> 关联文档：[StreamDock 逆向分析报告](docs/StreamDock逆向分析报告.md)（协议基线）

---

## 3.1 项目目录结构与功能说明

```
d:\Ai\N4PRO
├── n4pro-core\                     【核心功能】Rust 重制软件工程 【关联模块】全部 【修改权限】可新增/修改源码，禁止删除已上线模块
│   ├── Cargo.toml                  【核心功能】依赖与 release 优化配置 【关联模块】编译流程 【修改权限】可调整依赖
│   ├── build.rs                    【核心功能】调用 rc.exe 把 assets/app.ico 嵌入 exe 【关联模块】编译流程 【修改权限】可维护
│   ├── assets\app.ico              【核心功能】软件图标（官方图标提取，16~128 多尺寸） 【关联模块】build.rs/托盘 【修改权限】可替换
│   ├── build.bat                   【核心功能】一键 release 编译入口 【关联模块】编译流程 【修改权限】可维护
│   ├── src\
│   │   ├── main.rs                 【核心功能】参数解析、单实例、控制台/静默切换 【关联模块】app 【修改权限】可修改业务，禁止改动入口框架结构
│   │   ├── app.rs                  【核心功能】页面状态机、刷新、输入分派、动作效果 【关联模块】config/device/render/actions 【修改权限】可修改
│   │   ├── logging.rs              【核心功能】仅控制台日志输出（有界队列，不写文件） 【关联模块】全模块 【修改权限】可修改
│   │   ├── config\                 【核心功能】ini 与 manifest 解析（只读） 【关联模块】app 【修改权限】可新增字段，禁止写回配置
│   │   ├── device\                 【核心功能】HID 枚举/读写、协议帧、输入解析 【关联模块】app 【修改权限】协议层已实测，改动需回归测试
│   │   ├── render\                 【核心功能】112×112 按键、800×130 触摸条、GDI 文字、PNG 【关联模块】app 【修改权限】可优化视觉效果
│   │   ├── actions\                【核心功能】动作执行器（热键/打开/媒体/网络/鼠标/工具箱/操作流/音频/音频设备/触控条） 【关联模块】app 【修改权限】可扩展动作类型
│   │   └── platform\               【核心功能】Win32 FFI（HID/GDI/Shell/控制台/鼠标/剪贴板/CoreAudio/USB 监听） 【关联模块】全模块 【修改权限】仅在新增系统调用时更新
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
| 23 | 息屏/震动 | 设备息屏动作（亮度 0 切换）；震动协议尚未接入 | 可选 | device | 已上线 | 0.4.0 |
| 24 | 鼠标事件 | 单击/双击/移动/滚轮/拖拽，绝对坐标、修饰键、旋钮滚轮 | 按键/旋钮动作 | actions | 已上线 | 0.4.0 |
| 25 | 工具箱动作 | Super Hotkeys、Hotkey Switch、Text、Password、Close、Open APP | 按键动作 | actions | 已上线 | 0.4.0 |
| 26 | 操作流 | 多动作顺序执行 + 每个子动作 delay、循环、轮播、切换、旋钮分向 | 场景宏 | actions | 已上线 | 0.4.0 |
| 27 | 音频播放器 | MCI 播放/停止本地音频，音量、循环、淡入 | 按键动作 | actions | 已上线 | 0.4.0 |
| 28 | 切换音频设备 | Core Audio 枚举与默认播放/录音设备切换 | 按键动作 | actions | 已上线 | 0.4.0 |
| 29 | 触控条动作 | 通知中心/截图/锁屏/睡眠/媒体键/音量等 Windows 可用项 | 触控条 | actions | 已上线 | 0.4.0 |
| 30 | USB 自动重连 | 设备缺失/断开时关闭句柄、事件驱动等待接入并自动重新初始化 | 运行期 | platform/app | 已上线 | 0.4.0 |
| 31 | 控制台日志 | 仅控制台输出（独立线程 + 有界队列），不写文件；缓冲区固定 50 行 | 运行期 | logging | 已上线 | 0.5.0 |
| 32 | 系统托盘 | 启动隐藏到托盘、最小化进托盘、双击恢复、右键显示/退出 | 运行期 | platform | 已上线 | 0.5.0 |
| 33 | 软件图标 | 官方 ICO 多尺寸嵌入 exe，托盘与资源管理器显示 | 安装/运行 | build.rs | 已上线 | 0.5.0 |
| 34 | 启动提示 | 版本行、服务版参数说明、项目地址、设备信息 | 启动 | main | 已上线 | 0.5.0 |

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
  - `tx: Option<Sender<WriteMsg>>` HID 写队列（整页刷新批次可合并）
  - `dry_run/no_device/once: bool` 测试开关
- 核心方法：
  - `new(cfg, dry_run, dump_frames, no_device, once) -> Result<App, String>`：加载场景并定位页面
  - `run(&mut self)`：打开设备、发送初始化帧、进入事件循环
  - `refresh_all(&mut self)`：整页刷新（按批入队，写线程丢弃过期批次）
  - `emit / emit_refresh / emit_msg`：普通帧与整页刷新批次入队
  - `handle_report(&mut self, &[u8])`：解析并处理输入
  - `switch_page(&mut self, i32)` / `switch_scene(&mut self, &str)`
- 标准用法示例：
  ```rust
  let cfg = config::load(None)?;
  let mut app = App::new(cfg, true, false, false, true)?;
  app.run();
  ```
- 依赖关系：config、device、render、actions、platform
- 迭代记录：0.1.0 新建；0.3.8 写队列改为批次消息并合并过期整页刷新，退出帧改直写

### 类名称：HidDevice
- 所属文件：`n4pro-core/src/device/hid.rs`
- 核心功能：枚举并打开 N4 Pro 厂商 HID 集合，提供 1025B 写、512B 读。
- 核心属性：`handle: HANDLE`、`write_lock: Mutex<()>`（写帧互斥）、`output_len: usize`、`input_len: usize`
- 核心方法：`open_n4pro() -> Result<HidDevice, String>`、`write_frame(&[u8])`、`read_report(&mut [u8])`
- 标准用法示例：
  ```rust
  let dev = HidDevice::open_n4pro()?;
  dev.write_frame(&protocol::lig(60))?;
  ```
- 依赖关系：platform::win（SetupAPI/Hid/kernel32）
- 迭代记录：0.1.0 新建；0.3.8 增加写锁，避免写线程与退出回调的帧交错

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
- 核心属性：`uuid`、`name`、`controller`、`settings: Value`、`states: Vec<StateDef>`、`state_index`、`action_id`（实例 ID）、`children: Vec<ActionDef>`（多动作子动作）
- 核心方法：`state() -> StateDef`、`parse_action(&Value)`、`parse_state(&Value)`
- 标准用法示例：
  ```rust
  let def = parse_action(&json);
  let st = def.state();
  for child in &def.children { /* 多动作子项，含 Settings.delay */ }
  ```
- 依赖关系：serde_json
- 迭代记录：0.1.0 新建；0.4.0 增加 action_id 与多动作 children 解析

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

### 版本号：0.3.4
- 编译时间：2026-11-13 09:54
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：`refresh_touchbar()` 仅刷新触摸条；`max_scroll` 保证第 6 个按钮可完整滚出
  2. 修改：数据帧改为 0ms 连发（BAT 10ms、STP/CONNECT 30ms），提升跟手性；模式切换只重绘触摸条，不再重绘 10 键；按键按下不再触发图像刷新，仅切页整页重绘
  3. 删除：无
  4. 兼容说明：切页仍整页刷新（含 6~15 键）；单击不重绘

---

### 版本号：0.3.5
- 编译时间：2026-11-13 10:11
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：Control Bar 点击映射（0x39 返回 / 0x40~0x43 快捷位，按 X 命中 `(x+scroll-25)/152`）；启动恢复官方 `Settings.widgetCurrX`
  2. 修改：切页不再重置触摸条滚动位置；输入解析新增 `BarTap`
  3. 删除：无
  4. 兼容说明：只读配置；6~15 按键不受影响

---

### 版本号：0.3.6
- 编译时间：2026-11-13 10:20
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：无
  2. 修改：旋钮按下编码按官方映射修正：0x37→旋钮1、0x35→旋钮2、0x33→旋钮3、0x36→旋钮4；调试启动统一使用普通控制台模式
  3. 删除：无
  4. 兼容说明：旋钮旋转仍为 0xA1~0xA4；旋钮只触发动作，不重绘触摸条

---

### 版本号：0.3.7
- 编译时间：2026-11-13 10:28
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：退出信号 `CRT CLE` + 参数 `DC`（官方 closeHidDevice 关闭前发送）；控制台关闭/Ctrl+C 回调自动发送；旋钮旋转编码映射；空 ProfileUUID 的场景轮换
  2. 修改：旋钮旋转按 0xA0/0xA1、0x50/0x51、0x90/0x91、0x70/0x71 区分 1~4；多媒体编号 22 映射播放/暂停
  3. 删除：无
  4. 兼容说明：关闭窗口前会发送结束帧，避免设备 USB 异常；不写回配置

---

### 版本号：0.3.8
- 编译时间：2026-11-13 11:54
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：控制台日志独立线程输出（有界队列 256 行，写满丢弃）；写队列批次合并（整页刷新只发最新一批）；HID 写帧互斥锁；退出前停止写线程并直写结束帧（`--once`/`--probe` 退出也发送）
  2. 修改：日志文件名 `n4pro.log` → `N4Pro.log`（仍在 exe 运行目录 `logs\`）；Ctrl+C/Ctrl+Break 发送退帧后交回默认处理，确保进程退出；单实例互斥体改 `Local\` 并校验创建结果
  3. 删除：无
  4. 兼容说明：日志路径不变（仅文件名大小写）；渲染、协议帧、配置读取与按键/旋钮/触摸行为不变

---

### 版本号：0.4.0
- 编译时间：2026-11-14 00:22
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 9/9 通过
- 本次版本变更概要：
  1. 新增：鼠标事件（单击/双击/移动/滚轮/拖拽 + 旋钮）、工具箱（Super Hotkeys/Hotkey Switch/Text/Password/Close/Open APP）、操作流（多动作逐项延时/循环/轮播/切换/旋钮分向）、音频播放器（MCI）、切换音频设备（Core Audio）、触控条 Windows 可用项、USB 自动重连
  2. 修改：多动作子动作从动作对象 `Actions` 解析并保存 `ActionID`；动作执行增加触发方式（按下/旋转）；旋钮多媒体与音量按方向执行；设备缺失/断开不再退出
  3. 删除：无
  4. 兼容说明：配置只读；HID 协议帧与渲染不变；新增进程 DPI 感知以保证鼠标坐标一致

---

### 版本号：0.4.1
- 编译时间：2026-11-14 00:46
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：无
  2. 修改：按键动作绑定位置与图像渲染位置分离——物理上排按键读取配置 y2、物理下排读取配置 y1（图像显示保持现状）；按键日志增加“图像槽位/配置位置”
  3. 删除：无
  4. 兼容说明：仅影响按键动作绑定；旋钮、触摸条、整页渲染与协议帧不变

---

### 版本号：0.4.2
- 编译时间：2026-11-14 01:09
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 10/10 通过
- 本次版本变更概要：
  1. 新增：`Canvas::draw_fit`（等比完整放入目标框并居中；原图宽高未超过目标框时保持原尺寸、不放大）
  2. 修改：按键、触摸条、旋钮图标由 cover 裁剪改为 draw_fit，修复非方形/小图标被拉伸裁切
  3. 删除：`Canvas::draw_cover`（已无调用）
  4. 兼容说明：仅影响图标绘制，布局、文字层、HID 协议帧不变

---

### 版本号：0.4.3
- 编译时间：2026-11-14 01:20
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 10/10 通过
- 本次版本变更概要：
  1. 新增：无
  2. 修改：`draw_fit` 取消“小图不放大”分支，改为始终等比缩放完整放入目标框并居中（与官方 KeepAspectRatio 行为一致）；移除重复的 `draw_contain`
  3. 删除：`Canvas::draw_contain`（已被 draw_fit 取代）
  4. 兼容说明：仅影响图标绘制；小图标放大填满按键、非方形图保留比例并居中

---

### 版本号：0.4.4
- 编译时间：2026-11-14 01:32
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 10/10 通过
- 本次版本变更概要：
  1. 新增：`Scene.device_uuid/device_serial`；启动时按设备构建可用场景列表（manifest `DeviceUUID`/`DeviceSerialNumber` 过滤，创建时间排序）
  2. 修改：Scene Shift 只轮换本设备场景，不再切入其他设备（如 StreamDock[293S]）的空场景；启动日志增加“可用场景 N 个”
  3. 删除：无
  4. 兼容说明：无 DeviceUUID 的旧场景仍会纳入；页面/旋钮/渲染逻辑不变

---

### 版本号：0.4.5
- 编译时间：2026-11-14 01:43
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 10/10 通过
- 本次版本变更概要：
  1. 新增：无
  2. 修改：旋钮旋转方向归一化为 `+1`（奇数码/顺时针）与 `-1`（偶数码/逆时针），下游翻页、音量、多媒体、鼠标滚轮、轮播统一按正负判断
  3. 删除：无
  4. 兼容说明：修复顺/逆时针都执行同一方向的问题；日志“顺/逆时针”含义不变

---

### 版本号：0.5.0
- 编译时间：2026-11-14 02:05
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc，rc.exe 10.0.26100.0）
- 编译结果：编译成功；单元测试 10/10 通过
- 本次版本变更概要：
  1. 新增：系统托盘（启动隐藏、最小化进托盘、双击恢复、右键显示/退出）；`build.rs` + `assets/app.ico` 嵌入软件图标；控制台缓冲区固定 50 行
  2. 修改：取消文件日志与滚动，仅控制台输出；启动提示改为版本行、服务版参数说明、项目地址、设备信息（删除“日志文件”行）
  3. 删除：`N4Pro.log` 文件写入与日志滚动逻辑
  4. 兼容说明：HID、渲染、动作、配置读取不变；旧日志文件保留但不再写入

---

### 版本号：0.5.1
- 编译时间：2026-11-14 02:30
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：热键支持左右修饰键（KeyCtrl/RKeyCtrl、KeyShift/RKeyShift、KeyOption/RKeyOption、KeyCmd/RKeyCmd）；支持 `VKeyCodes` 多主键同时按下
  2. 修改：热键统一改为扫描码发送（SendInput + KEYEVENTF_SCANCODE），修饰键按下/主键按下/释放之间增加 15~18ms 间隔；Alt 正确读取 `KeyOption`
  3. 删除：无
  4. 兼容说明：修复左Ctrl+F10、左Ctrl+左Shift+F6 及单键在部分程序/游戏中不生效的问题

---

### 版本号：0.5.2
- 编译时间：2026-11-14 02:47
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：`ITaskbarList::DeleteTab/AddTab`，隐藏/恢复控制台时同步增删任务栏按钮（修复 Win11 托盘后任务栏残留）
  2. 修改：Ctrl+C / Ctrl+Break 改为收进托盘不退出；关闭控制台窗口发送结束帧后自动重启到托盘；单实例互斥改为等待旧实例退出（最多 6 秒）
  3. 删除：无
  4. 兼容说明：只有托盘右键“退出”或系统注销/关机才真正结束进程；Win10 行为不变

---

### 版本号：0.5.3
- 编译时间：2026-11-14 02:57
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：无
  2. 修改：从托盘恢复控制台时不再手动 `AddTab`（由系统随窗口显示自动恢复）；隐藏时 `DeleteTab` 后再以 80/200/400ms 补删，处理 Win11 延迟重建任务栏按钮；移除对控制台窗口无效的 DWM cloak
  3. 删除：DWM 隐身调用
  4. 兼容说明：修复 Win11“首次启动无残留，但从托盘恢复后再最小化/关闭又出现任务栏图标”的问题

---

### 版本号：0.6.0
- 编译时间：2026-11-14 03:25
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：`src/ui.rs` 日志窗口 + 系统托盘；GUI 子系统（release 不创建控制台）；`--console` 调试模式
  2. 修改：日志改为内存 50 行环形缓冲并推送到日志窗口；关闭/最小化窗口仅隐藏；托盘退出发送 `CLE+DC`；启动提示保持版本/服务参数/项目地址三行
  3. 删除：控制台窗口相关逻辑（50 行缓冲区、任务栏 DeleteTab、DWM 尝试）、`platform/tray.rs`
  4. 兼容说明：Win10/Win11 不再出现控制台任务栏残留；HID、渲染、动作、USB 自动重连不受影响

---

### 版本号：0.6.1
- 编译时间：2026-11-14 05:14
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：无
  2. 修改：日志窗口输出前把 LF 统一转换为 CRLF（Win32 EDIT 控件只识别 CRLF），修复日志挤在一行的问题
  3. 删除：无
  4. 兼容说明：复制出的日志与窗口显示排版一致

---

## 最新版本对比变更概要

### 对比版本：0.6.1 VS 0.6.0
- 对比生成时间：2026-11-14 05:15
- 新增内容：无
- 修改内容：日志窗口换行改为 CRLF
- 删除内容：无
- 核心影响说明：只影响日志显示排版，其他功能不变
