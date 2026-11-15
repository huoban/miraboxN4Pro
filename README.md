# 有问题别问！AI全自动。

# miraboxN4Pro
- 妙联宝N4Pro官方软件内存严重泄漏。逆向后制作的简易替代品
- Rust 代码，全部内存不泄漏。
- 使用官方StreamDock.exe生成场景与快件后。本软件启动后调用官方场景。（仅重制了基础功能，如快件组合等。插件功能未适配因为别人不需要。。自己折腾把）
- 去除动态背景，背景只使用纯黑。（浪费通信没意义）
- 服务版启动参数：n4pro-core.exe -serve（静默后台，无控制台）
- 通信协议在机器flash内，软件中的固件升级包不带 factory 区,是增量升级不修改通信协议。而且通信协议疑似是通用的。

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
  5. 0.6.2 回退重建（2026-11-14）：按用户确认方式 A 恢复 0.6.2 基线源码并重新编译（测试 10/10 通过），规避 0.7.x 真机回归；0.7.0/0.7.1 保留在 git 标签中。
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

### 版本号：0.6.2
- 编译时间：2026-11-14 06:11
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：无
  2. 修改：软件图标更换为用户提供的 `54cdfe1ef082da8bfe8a23734c7ddca9.png`，转换为 16/24/32/48/64/128/256 多尺寸 `assets/app.ico` 并重新嵌入 exe；托盘与窗口图标随资源 ID=1 自动更新
  3. 删除：无
  4. 兼容说明：仅图标资源变化，功能不受影响

---

### 版本号：0.7.0
- 编译时间：2026-11-14 07:08
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功
- 本次版本变更概要：
  1. 新增：渲染并行化（`render::render_parallel`，2 线程瞬时全速）；图像缓存 LRU 上限 48 项 + 滚动类条目仅保留最近 2 项；按键映射单元测试
  2. 修改：PNG 压缩由 Fast 调优为 Best（实测同页 Fast 846 帧 / Best 375 帧，总耗时最短）；主循环空闲改阻塞接收（事件驱动零唤醒）；整页刷新改脏标记合并（快速连续切页只渲染最新状态）；写线程首错即中止整批（断连不再滞留）；多状态按键按下/抬起重绘该键；文字层 DIB 只按文字矩形分配 + HFONT 缓存复用；BAT 帧节流 10ms→5ms；刷新组帧命中缓存项零拷贝
  3. 删除：`key_pos`（被 `key_action_pos` 取代后无调用）
  4. 兼容说明：HID 协议帧内容、动作执行、配置读取均不变；渲染像素结果一致

---

### 版本号：0.7.1
- 编译时间：2026-11-14 08:07
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 14/14 通过
- 本次版本变更概要：
  1. 新增：无
  2. 修改：修复 0.7.0 回归「快捷匹配错位」——`key_slot` 按用户实测标定重写（上排输入 1~5 → 槽位 11~15，下排输入 6~10 → 槽位 6~10），图像与动作统一使用 `key_action_pos`（上排 → 配置 y2，下排 → 配置 y1），全量刷新/多状态键重绘/日志槽位号三处一致；修复 0.7.0 回归「旋钮模式丢失全部 UI」——写线程恢复 0.6.2 容错（单帧写失败仅跳过并退避 200ms 继续，不再中止整批），BAT 节流 5ms 回退 10ms，避免长批次因一帧超时整屏黑
  3. 删除：无
  4. 兼容说明：键位图像与动作执行对齐用户标定表（槽位 6~10=下排/配置 0,1~4,1，槽位 11~15=上排/配置 0,2~4,2）；其余 0.7.0 优化（并行渲染、LRU 缓存、脏标记合并、阻塞接收、PNG Best）全部保留

---

### 版本号：0.6.2（回退重建）
- 编译时间：2026-11-14 08:24
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 10/10 通过
- 本次版本变更概要：
  1. 新增：无
  2. 修改：按用户确认「方式 A」将 `n4pro-core/src`、`Cargo.toml`、`Cargo.lock` 全部恢复为 `v0.6.2` 标签基线，撤销 0.7.0 与 0.7.1 的代码改动；恢复 `key_pos` 与 `key_slot = key + 5` 旧映射、`refresh_all` 同步渲染、写线程单帧失败退避 200ms 继续、BAT 帧 10ms 节流
  3. 删除：0.7.x 引入的 `render::render_parallel`、ImageCache 的 LRU/volatile 淘汰、`pending_refresh` 脏标记合并刷新、阻塞接收事件循环、PNG Best 压缩、多状态按键重绘、app 模块单元测试（均随源码回退一并移除）
  4. 兼容说明：行为与真机已验证的 0.6.2 一致；0.7.0 / 0.7.1 提交与标签仍完整保留在 git 历史中，可随时恢复或对照

---

### 版本号：0.6.3
- 编译时间：2026-11-14 16:37
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 11/11 通过
- 本次版本变更概要：
  1. 新增：触摸条轻点兜底（`finish_bar_tap_fallback`）——固件在感应区分界（x≈400）附近点击时只上报触摸按下/抬起、不生成 BarTap 帧，此时按触点坐标与按住时长（≤12px、≤600ms）本地判定轻点并执行对应条目，日志标记「触摸兜底点击」；`InputEvent::TouchDown/TouchUp` 增加 x 坐标字段
  2. 修改：触摸抬起优先使用抬起帧坐标做命中判定；收到 BarTap 帧时置 `bar_tap_seen`，本次触摸不再走兜底，避免重复执行；Control Bar 抬手吸附逻辑不变
  3. 删除：无
  4. 兼容说明：旋钮模式、滑动吸附、物理按键、渲染均不受影响；错位按下帧（`idx=1 + 非零坐标`）仍按原逻辑解析，未做修正（待物理按键原始帧对照）

---

### 版本号：0.6.4
- 编译时间：2026-11-16 01:40
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 12/12 通过
- 本次版本变更概要：
  1. 新增：`SETLB` 携带 4 个旋钮键标题颜色（官方抓包实测 `[10..21]` 为四组 RGB，默认 `#ff0000/#00aa00/#ffff7f/#0055ff`，来自旋钮 1~4 键的 `TitleColor`）；单键刷新 `refresh_key`（BAT→STP）；多状态键按下/抬起立即刷新该键
  2. 修改：`BGPIC` 长度字段由 `[11..13]` 修正为官方格式 `[10..12]`（24 位大端），`[13..16]` 保留为偏移（整图发送为 0），修复触摸条背景图错位导致的半显示/花屏；`BGCLE` 参数由 `[10]` 移到官方位置 `[11]`
  3. 删除：无
  4. 兼容说明：BAT 帧格式（长度大端 `[10..11]`、槽位 `[12]`）经抓包核对本来就正确，未改；按键槽位、动作执行、输入解析、旋钮模式渲染不受影响

---

---

### 版本号：0.6.5
- 编译时间：2026-11-16 01:58
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 12/12 通过
- 本次版本变更概要：
  1. 新增：`Canvas::encode_png` 改为 PNG-8 调色板编码（颜色 ≤256 用精确调色板，超过则 3-3-2 量化，压缩用 `png` crate `Compression::High`）；超过 65535 字节时自动降色重编码，仍超限则拒绝发送并记录错误
  2. 修改：触摸条背景图体积由 97,767 字节降至 11,037 字节，杜绝 BGPIC 16 位长度截断导致的半显示/花屏；`Cargo.toml` 增加 `png = "0.18"` 依赖
  3. 删除：无
  4. 兼容说明：图像内容与尺寸不变，仅编码方式变化；官方抓包证实全部图像为 PNG，单帧最大 27,631 字节，重制版体积已与官方同量级

---

---

### 版本号：0.6.6
- 编译时间：2026-11-16 02:27
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 12/12 通过
- 本次版本变更概要：
  1. 新增：`src/pool.rs` 动作线程池——并发上限 15、最少 1 个常驻、空闲超过 3 个自动退出、单线程空闲 24 小时回收
  2. 修改：`on_key` 改为提交线程池执行，不再阻塞主循环；同一 `action_id` 执行期间重复触发一律忽略；动作效果带页面代次回传，切页后迟到的效果丢弃；主循环超时由 100ms 改为 20ms 并处理回传效果
  3. 删除：主线程同步执行动作的旧路径
  4. 兼容说明：换页后同位置是新 `action_id`，会独立并发执行；不同动作互不阻塞；动作日志顺序可能交错，单动作内部顺序不变；长动作退出时进程直接结束（不等待 30 秒）

---

---

### 版本号：0.6.7
- 编译时间：2026-11-16 02:36
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc）
- 编译结果：编译成功；单元测试 12/12 通过
- 本次版本变更概要：
  1. 新增：无
  2. 修改：动作线程池单线程空闲回收时间 24 小时 → **10 秒**；空闲检查间隔 60 秒 → 2 秒
  3. 删除：无
  4. 兼容说明：最少 1 个常驻线程、空闲超过 3 个立即退出、并发上限 15 均不变；执行中的长动作不受影响

---

---

### 版本号：0.6.8
- 编译时间：2026-11-16 03:03
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc，本次以 `cargo build --release` 执行）
- 编译结果：编译成功；单元测试 12/12 通过
- 本次版本变更概要：
  1. 新增：无
  2. 修改：`BGPIC` 长度字段由 0.6.4 的 `[10..12]` 恢复为官方抓包实测的 `[11..13]`（`[10]=0x00`、`[14..17]` 为偏移），修复固件按 256 倍长度读取并吞掉后续 BAT/STP 帧导致的花屏、图标丢失与模式切换不刷新；`Canvas::encode_png` 由 PNG-8 索引色回退为官方同款 RGBA（color type 6）+ `Default` 压缩 + `Adaptive` 扫描行过滤；`Cargo.toml` 移除 `png = "0.18"` 直接依赖
  3. 删除：`encode_png_indexed`、`write_indexed_png` 调色板编码路径及 65535 字节降色逻辑
  4. 兼容说明：与 0.6.3 的 BGPIC 布局一致，BAT 头（长度 `[10..11]`、槽位 `[12]`）不变；官方 BGPIC 实测最大 77,707 字节，重制版触摸条约 70~85 KB，属官方同量级；动作线程池、输入解析、切页逻辑不受影响

---

---

### 版本号：0.6.9
- 编译时间：2026-11-16 03:28
- 环境配置：`d:\Ai\N4PRO\n4pro-core\build.bat`（Rust 1.95.0 x86_64-msvc，本次以 `cargo build --release` 执行）
- 编译结果：编译成功；单元测试 14/14 通过
- 本次版本变更概要：
  1. 新增：`Canvas` 绘制裁剪区 `set_clip/clear_clip`；触摸条测试「静止时显示五个完整条目」「滚动时两侧条目按边界裁切」
  2. 修改：Control Bar 条目改为在 x=25~775 安全区内连续绘制，半可见条目按边界裁切（与官方抓包逐像素一致），不再整块跳过；删除官方不存在的左右箭头与深色圆角容器；`render_control_bar` 去掉 `show_left/show_right` 参数
  3. 删除：Control Bar 的箭头绘制与容器背景
  4. 兼容说明：命中公式 `(X+scroll-25)/152` 不变，滚动中两侧半显示条目仍可点击（最多 6 个条目响应，与官方一致）；抬手吸附 152 网格不变；按键、旋钮模式不受影响

---

## 最新版本对比变更概要

### 对比版本：0.6.9 VS 0.6.8
- 对比生成时间：2026-11-16 03:28
- 新增内容：`Canvas` 裁剪区 API；2 个触摸条布局单元测试
- 修改内容：Control Bar 滚动时两侧条目按 25/775 边界裁切显示；移除箭头与深色容器
- 删除内容：Control Bar 左右箭头绘制、深色圆角容器背景
- 核心影响说明：左右滑动时边缘不再出现整块空白，半显示图标与官方效果一致；滚动吸附、命中与动作执行不变
---

---

### 版本号：1.0.1
# N4 Pro Core v0.6.9 代码质量审核报告

> 审核对象：`n4pro-core`（Rust，36 个源文件，约 2600 行）
> 审核方式：逐文件人工审读 + `cargo clippy --all-targets` 静态检查
> 编译结果：**0 error**，66 个 clippy warning（多为命名/风格类）
> 结论：整体架构清晰（读写线程分离、动作线程池、代次丢弃迟到效果等设计合理），无内存安全（unsafe 越界/悬垂）类崩溃缺陷；但存在 **3 处逻辑接线缺陷** 和 **若干资源/内存增长风险**，需重点关注。

---

## 一、高危 / 中危：逻辑缺陷

### 1.1 「进入子场景」动作接线错误，`Effect::SwitchScene` 为死代码

**文件**：`src/actions/profile.rs`、`src/actions/mod.rs`、`src/app.rs`

`profile.openchild`（进入子场景）本应触发场景切换，但实现返回的是**页面跳转**效果：

```rust
// src/actions/profile.rs:11-23
if uuid.contains("profile.openchild") {
    let target = s_str(settings, &["ProfileUUID", "profileUUID", ...]).unwrap_or_default();
    ...
    return Effect::GotoPage(target);   // ❌ 应为 Effect::SwitchScene(target)
}
```

而 `Effect::SwitchScene(String)`（`src/actions/mod.rs:39`）**定义了却从未被构造**（clippy 已确认 `variant SwitchScene is never constructed`）。`app.rs` 里对应处理分支 `Effect::SwitchScene(uuid) => self.switch_scene(&uuid)`（`src/app.rs:986`）以及 `switch_scene()` 整段实现（`src/app.rs:1058-1079`）因此全部是死代码。

**影响**：`GotoPage` 会把 `ProfileUUID` 当作「页码」或「页面名」在当前场景的 `pages` 里查找，必然落到 `未找到目标页面` 分支，**进入子场景功能完全失效**。

**修复建议**：改为返回 `Effect::SwitchScene(target)`；同时确认 `switch_scene` 中 `parent.join(uuid)`（`src/app.rs:1063`）的 `uuid` 语义——当前代码假设 `uuid` 就是 `profiles/` 下的目录名，需与 manifest 中 `ProfileUUID` 的实际取值对齐。

---

### 1.2 音频设备切换：COM 未初始化即调用 `CoCreateInstance`

**文件**：`src/actions/audio_device.rs`

`switch()` 中，`radio == 0`（在两台设备间切换）时，先读取当前默认设备，再做 COM 初始化：

```rust
// src/actions/audio_device.rs:157-160
if radio == 0 && !select2.is_empty() {
    let current = default_device_id(flow).unwrap_or_default();  // ❌ 此处已 CoCreateInstance
    ...
}
// ...
let hr = unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED) };  // 180 行，初始化在后面
let ok = set_default_device(&target);
```

`default_device_id()` 内部直接调用 `CoCreateInstance`（`src/actions/audio_device.rs:93`），但动作池线程此时**尚未 `CoInitializeEx`**。在未初始化 COM 的线程上调用会返回 `CO_E_NOTINITIALIZED (0x800401F0)`。

**影响**：`radio=0` 的「双设备来回切换」始终读不到当前设备，`current` 恒为空，最终恒等于固定切到 `select`，**切换（toggle）语义失效**（`radio!=0` 固定切换不受影响）。

**修复建议**：把 `CoInitializeEx` 提前到 `default_device_id` 调用之前，并保证 `CoUninitialize` 与成功次数配对。

---

### 1.3 输入解析：`GestureEnd` 死代码，`0xB1` 语义矛盾

**文件**：`src/device/input.rs`、`src/app.rs`

- 文件头注释写「手势: 0xB1 触摸结束」（`src/device/input.rs:6`），但解析代码把 `0xB1/0xB2` 映射为**模式切换**：

```rust
// src/device/input.rs:67-68
0xB1 => InputEvent::ModeChanged { control_bar: false },
0xB2 => InputEvent::ModeChanged { control_bar: true },
```

- `InputEvent::GestureEnd`（`src/device/input.rs:15`）**从未被构造**（clippy 确认），`app.rs` 中 `InputEvent::GestureEnd => self.finish_touch()`（`src/app.rs:679`）是死代码。

**影响**：触摸结束目前完全依赖 `TouchUp`（index=0,state=0）路径。若固件在部分场景只上报 `0xB1` 手势结束帧而不上报 `TouchUp`，则触摸结束不会被识别（`touch_active` 持续为真，后续 `TouchMove`/点击判定异常）。这是「注释与实现二选一必错」的不一致，需对照实机抓包确认 `0xB1` 的真实含义。

**修复建议**：若 `0xB1` 实为触摸结束，改回 `GestureEnd` 并把模式切换移到其他编码；若实为模式切换，删除 `GestureEnd` 变体与其处理分支，修正注释。

---

## 二、内存 / 资源泄漏风险

> Rust 自身内存安全，此处「泄漏」指：无界缓存增长、GDI 句柄未释放、全局列表只增不减、资源延迟释放等。

### 2.1 `ImageCache` 在 Control Bar 拖动期间无界增长

**文件**：`src/app.rs:570-581`、`src/render/cache.rs`

`bar_png()` 的缓存键含 `self.scroll`：

```rust
let sig = format!("bar|{}|{}|{}", self.scene.dir.display(), self.scroll, self.page_idx);
```

`ImageCache` 是 `HashMap<String, Vec<u8>>`，仅在**切页 / 切模式**时 `clear()`。拖动滚动条时 `on_touch_drag` 以 60ms 节流不断写入新的 `scroll` 值（未经 152px 吸附），每次生成一张完整 800×130 PNG 并缓存。**同一页面内反复拖动会累积数十至上百张 PNG（约数十 KB/张），且不回收**。

**修复建议**：为缓存设置容量上限（LRU/简单 `len()` 阈值）或在滚动时按「吸附后的格子」作键、限定缓存条目数。

### 2.2 Soundboard 全局别名列表只增不减

**文件**：`src/actions/soundboard.rs:82-84`、`src/actions/soundboard.rs:102-116`

`play()` 每次把 alias push 进 `ALIASES`；只有 `stop_all()` 会清空。若音频**自然播放结束**（未触发「停止全部」），对应 alias 字符串永久留在 `Vec<String>` 中。长时间运行会缓慢累积。

**修复建议**：监听播放完成事件移除 alias，或改用「打开→播放→关闭」的一次性 alias（非 repeat 的播放结束后主动 `close` 并出队）。

### 2.3 GDI 对象未释放（画刷 / 字体 / 图标）

**文件**：`src/ui.rs`

- `CreateSolidBrush(0x001E1E1E)`（`src/ui.rs:223`）存入 `BG_BRUSH`，从未 `DeleteObject`；
- `CreateFontW(...)`（`src/ui.rs:283`）返回的字体从未 `DeleteObject`；
- `LoadIconW` 图标未释放（系统图标可接受，自有图标需 `DestroyIcon`）。

窗口为进程级常驻，泄漏量固定（各 1 个），**属轻微**，但不符合规范。建议在 `WM_DESTROY` / 退出路径释放。

### 2.4 剪贴板写入失败路径泄漏全局内存

**文件**：`src/platform/win.rs:629-653`

`clipboard_set_text` 中，`SetClipboardData` 的返回值未检查；若设置失败，已 `GlobalAlloc` 的 `mem` 既未被系统接管也未被 `GlobalFree`，发生一次 `GMEM_MOVEABLE` 泄漏。概率极低，但属遗漏。

### 2.5 会话结束后的线程 / 设备句柄延迟释放

**文件**：`src/app.rs:371-381`、`src/app.rs:464-472`

心跳线程持有 `write_tx`（`Sender<WriteMsg>`）的一个 clone，`end_session()` 仅把 `self.tx = None` 丢弃主线程持有的那份。心跳线程每 2 秒才醒来检查 `SESSION_RUNNING`，因此**写线程（及其持有的 `Arc<HidDevice>` 设备句柄）在会话结束后最多再存活约 2 秒**才随 channel 断开退出。重连场景下旧句柄延迟释放。

**修复建议**：心跳线程同时监听 `write_rx` 断开（改用 `send` 失败即退出已覆盖）；可显式让写线程在收到断开标记时立即 `drop`，或缩短心跳判定周期。

---

## 三、并发 / 健壮性

### 3.1 多动作 `run_children` 只保留最后一个页面效果

**文件**：`src/actions/multi.rs:51-61`

```rust
for child in children {
    ...
    let e = execute(child, ...);
    if e != Effect::None { effect = e; }   // 覆盖式，前面的效果丢失
}
```

若一个 routine 内含「下一页 + 下一页」两个子动作，最终只返回一次 `NextPage`，**只翻一页**。同理，多个页面/场景类子动作会互相覆盖。

**修复建议**：返回 `Vec<Effect>` 或让 `Effect` 支持叠加；至少对页面导航类做累计。

### 3.2 `routine` 无限循环上限仍偏大

**文件**：`src/actions/multi.rs:79-84`

`cycleMode=1` 的无限循环被硬限制为 1000 轮。若子动作含长延时（实测可到 230s），1000 轮可持续数小时，长期占用一个动作池线程。上限方向正确，但建议配合总时长上限或可取消机制。

### 3.3 `EXITING` 标志不复位

**文件**：`src/app.rs:51`、`src/app.rs:72`

`EXITING` 置位后永不复位。`write_frames` 见其为真即停止发送。对「退出」路径无害，但 `send_exit_frame_from_handler` 也被 `--probe` / `--once` 复用，若未来在会话中途触发而未退出进程，将永久停发帧。建议改为会话级状态或显式复位。

### 3.4 写线程 `Refresh` 合并会丢弃中间普通帧

**文件**：`src/app.rs:303-321`

处理 `WriteMsg::Refresh` 时 `try_recv` 排空队列：遇到新的 `Refresh` 会 `tail.clear()`，导致两次整页刷新**之间**的普通帧（如心跳 `RGBS`、`CONNECT`）被一并丢弃。当前丢弃的主要是心跳帧，影响不大，但逻辑上「保留刷新批次之后的帧」这一注释与实际（清空 tail）不符，需留意。

### 3.5 读线程 `INFINITE` 阻塞（理论风险）

**文件**：`src/device/hid.rs:233`

`read_report` 用 `WaitForSingleObject(..., INFINITE)`。若设备拔出时挂起的重叠读未完成（极端固件行为），读线程将永久阻塞，且其持有的 `Arc<HidDevice>` 不会释放（`Drop` 无法执行，句柄泄漏）。正常情况下 HID 拔出会完成挂起读，此条为兜底建议：改用有限超时 + 句柄失效检测。

---

## 四、死代码 / 未使用项（clippy 确认）

| 位置 | 项 | 说明 |
|---|---|---|
| `src/actions/mod.rs:26` | `Trigger::LongPress` | 从未构造，长按未实现 |
| `src/actions/mod.rs:39` | `Effect::SwitchScene` | 见 §1.1，接线缺失 |
| `src/device/input.rs:15` | `InputEvent::GestureEnd` | 见 §1.3 |
| `src/device/protocol.rs:56/67` | `lblig()` / `qucmd()` | 未使用 |
| `src/config/manifest.rs:44` | `ActionDef.controller` | 解析后从未读取 |
| `src/config/manifest.rs:72-74` | `Scene.device_uuid/device_serial` | 从未读取（`build_scene_list` 直接重读 manifest） |
| `src/config/resolve.rs:9` | `AppConfig.ini_path` | 从未读取 |
| `src/config/ini.rs:58/63` | `section_exists/section_names` | 未使用 |
| `src/config/mod.rs:7`、`src/device/mod.rs:8`、`src/render/mod.rs:11`、`src/ui.rs:7` | 多余 `pub use` | `StateDef` / `InputEvent` / `Canvas` / `SyncSender` |
| `src/pool.rs:65` | `idle_since` 初值 | 立即被 `pool.rs:70` 覆盖，冗余赋值 |

---

## 五、代码风格（clippy 66 项 warning）

主要类别（均可由 `cargo clippy --fix` 部分自动修复）：

- **命名规范**：`win.rs` 中大量 Win32 类型名（`ULONG_PTR`、`WNDPROC`、`WNDCLASSW`、`POINT`、`MSG`、`NOTIFYICONDATAW`）触发 upper-case-acronym 警告（FFI 绑定属预期，可加 `#[allow]`）；测试函数中文名触发 `non_snake_case`。
- **参数过多**：`draw_text`（9/7）、`draw_text_rect`（13/7）。
- **可读性**：`ui.rs:103` 的 `loop { match rx.try_recv() }` 可写为 `while let`；`ui.rs:226` 的 `1usize as *const u16` 建议 `std::ptr::dangling`；`canvas.rs:49` 循环可用迭代器。
- **冗余**：`run_hidden` 中 `STARTUPINFO` 先用 `Default` 再逐字段赋值。

---

## 六、总体结论

**优点**：
- 线程模型清晰（HID 读/写独立常驻 + 动作线程池），长动作不阻塞输入；
- 「页面代次 + 迟到效果丢弃」机制避免了切页竞态；
- `unsafe` FFI 结构体布局（`OVERLAPPED`、`GUID`、`HIDP_CAPS`、`INPUT` 等）经核对与 Win32 定义一致，未见内存安全缺陷；
- 输入/协议/渲染模块均有单元测试，且 `cargo clippy` 通过（0 error）。

**必须修复（建议优先级 P0-P1）**：
1. `profile.openchild` → `Effect::SwitchScene` 接线（§1.1）；
2. 音频切换 COM 初始化顺序（§1.2）；
3. `0xB1` 语义 / `GestureEnd` 死代码澄清（§1.3）。

**建议修复（P2）**：
- `ImageCache` 容量上限（§2.1）；
- Soundboard 别名回收（§2.2）；
- `run_children` 效果叠加（§3.1）；
- GDI 对象与剪贴板失败路径释放（§2.3 / §2.4）。

其余为健壮性加固与代码整洁项，可按排期逐步清理（`cargo clippy --fix` 可先消化 7 处）。

---

### 版本号：1.0.2
- 编译时间：2026-11-16 05:35
- 环境配置：`D:\Ai\CodeBuddy\N4-pro\n4pro-core`（Rust x86_64-msvc，以 `cargo build --release` 执行）
- 编译结果：编译成功；单元测试 14/14 通过
- 本次版本变更概要：
  1. 新增：无（本次为缺陷修复与健壮性完善）
  2. 修改：
     - 热键 Win 修饰键：字段读取由 `KeyCmd`/`RKeyCmd` 改为文档 05 的 `KeyWin`/`RKeyWin`（同时兼容旧写法），修复含 Win 键组合键不生效
     - 普通热键支持 `executionTimes` 重复执行次数（上限 20，与 Super Hotkeys 一致）
     - HID 写超时后调用 `CancelIo` 取消挂起的重叠写，消除栈上 `OVERLAPPED` 被异步回写的未定义行为风险
     - 修复 USB 等待中 `MsgWaitForMultipleObjects` 返回值判断（`r == 1` → `r == 0`），事件驱动唤醒路径恢复生效
     - `BAT` 头帧显式写入分片序号字段（载荷 `[13]`，整图单次发送为 0，与官方抓包一致）
     - 写队列由无界 `channel` 改为有界 `sync_channel`（容量 8），满则丢弃新帧，心跳改 `try_send`，避免设备卡顿/断连时帧无界积压
     - `ImageCache` 由「满即整体清空」改为 LRU 淘汰，避免 Control Bar 拖动时整表清空导致全量重渲染
     - `run_offline` 离线模式不再死循环，收到退出请求即返回
     - 托盘「退出」改走 `graceful_exit`：发送 `CLE+DC` 后请求主循环优雅退出，统一走 `end_session` 收尾（替代直接 `process::exit`）；USB 等待态也可被退出请求打断
     - release 优化级别 `opt-level` 由 `z` 改为 `2`（体积换速度，缓解 PNG 编码/缩放的整页刷新与拖动热点）
     - 为 6 个含中文测试函数名的 `mod tests` 加 `#[allow(non_snake_case)]`，消除命名告警
  3. 删除：无
  4. 兼容说明：
     - 退出帧 `"DC"` 位置（载荷偏移 1 字节之疑）经评审确认忽略，保持现状不改动
     - `ureq` 依赖保留默认 `tls` 特性：HTTP 动作的 URL 可能为 `https`，需 TLS 支持；正文 `gzip` 解压能力未使用但不影响
     - 协议帧布局（DIS/LIG/BAT/BGPIC/CLE/SETLB/STP/CONNECT）、按键槽位映射、触摸条命中公式均未改动

---
