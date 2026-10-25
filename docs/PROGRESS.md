# 2.0 进度日志

只追加，不改旧行。每次推送前由 `scripts/checkpoint.sh` 写入一行。中断后从最后一行的「下一步」继续。

| 时间 (UTC+8) | 阶段 | 完成了什么 | 下一步 |
|---|---|---|---|
| 2026-10-24 18:10 | 规划 | 建立 v2 分支，提交 PLAN / DECISIONS / PROGRESS 与 checkpoint 脚本 | 等待第 7 节三项拍板，然后开始 M0 骨架 |
| 2026-10-24 18:18 | 规划 | 方案补充：常驻托盘、右键控制面板、四语种（1.10/1.11 已在 Mac 版实现） | 等待第 7 节剩余两项拍板（Windows 服务、仓库），然后开始 M0 骨架 |
| 2026-10-24 18:40 | 规划 | 确认 Windows 只读索引服务（D-005） | 确定新仓库名称并建仓，然后开始 M0 骨架 |
| 2026-10-24 18:44 | 规划 | 新建仓库 luckySpro/Jseek，迁入方案、决策（含 D-006）与进度日志 | 在 dev 分支开始 M0 骨架：Rust workspace + Tauri 2 + Vue 3，三平台 CI 空壳 |
| 2026-10-24 18:49 | M0 | 骨架：Rust workspace（jseek-core、jseek-platform）+ Tauri 2 + Vue 3 空壳（暖纸风格、四语种）+ 三平台 CI | 等 CI 结果：修复编译问题，确认 macOS arm64/x64、Windows x64 均产出安装包 |
| 2026-10-24 18:55 | M0 | CI：macOS arm64/x64 构建成功，锁文件已由 CI 提交；失败时把构建日志尾部写成注释 | 查明并修复 Windows 构建失败 |
| 2026-10-24 19:01 | M0 | Windows：查明 MSI 失败原因是 WiX 不接受发布者名中的 &，改为 NSIS 用原名、MSI 单独用转义后的名字构建 | 确认三平台 CI 全绿，M0 收尾 |
| 2026-10-24 19:06 | M0 | Windows NSIS 安装包构建成功；MSI 的发布者覆盖改用配置文件（cmd 会在 & 处截断命令行参数） | 确认 MSI 构建通过，三平台全绿后 M0 收尾 |
| 2026-10-24 19:13 | M0 ✅ | M0 完成：三平台 CI 全绿（macOS arm64/x64 DMG、Windows x64 NSIS+MSI，核心测试与 clippy 通过）；记录 D-007 | M1：C++ 引擎去 GCD/CoreFoundation 依赖，三平台跑通原有 11,962 个测试 |
| 2026-10-24 20:25 | M1 | 引擎原样迁入 engine/（含 85 个测试模块与 test_all），CI 新增 macOS 引擎测试作为基线 | 确认 macOS 上 11,962 个测试全过；然后写跨平台层（并行调度、Unicode 小写） |
| 2026-10-24 20:33 | M1 | 去 GCD/CoreFoundation：新增 Concurrency.h（parallelFor、SerialQueue、RepeatingTimer）与可移植 Unicode 小写；新增 Part 77 测试（本地 ASan/UBSan/TSan 通过，并修复一处等待中的悬垂引用） | 确认 macOS 上 11,962+17 个测试仍全过；然后加 CMake，在 Linux/Windows 编译并跑引擎测试 |
| 2026-10-24 20:36 | M1 | 可移植扫描器（Linux 用 POSIX，Windows 用 std::filesystem）；Mac 专属模块与测试条件编译；测试辅助函数跨平台；F_NOCACHE 条件化；CMake 构建；CI 新增 engine-linux 与 engine-windows（后者暂不阻塞） | 根据 CI 修复 Linux 编译/测试；收集 Windows 错误清单 |
| 2026-10-24 20:41 | M1 | 修复：Makefile 加 -I.；GCC 不接受局部变量作默认参数（改 static constexpr）；修正 Windows 报错过滤规则 | 确认 macOS/Linux 引擎测试通过；拿到 Windows 错误清单 |
| 2026-10-24 20:47 | M1 | Windows 兼容层 Compat.h/.cpp（unistd、fsync、access、localtime_r；replaceFile 用 MoveFileExW 保证原子替换）；修复 u8string、GCC 默认参数；放宽定时器测试 | 看 macOS/Linux 是否全绿；继续处理 Windows 编译错误 |
| 2026-10-24 20:52 | M1 | Linux 引擎测试全绿（11,910 通过）；Mac 定时器测试已过；新增 Builtins.h（MSVC 的 ctz/prefetch 与 memmem） | 继续处理 Windows 编译错误；复查 Mac 上 batchRescanPrefix 性能测试是否为波动 |
| 2026-10-24 20:57 | M1 | macOS 11,979 / Linux 11,910 全部通过；Windows 引擎库已能编译。修复 Windows 测试代码：ftruncate/truncate 兼容、fs::path 转字符串、守护进程测试（fork）仅限 POSIX | 继续 Windows：编译通过后看测试结果 |
| 2026-10-24 21:02 | M1 | Windows：utime/popen/pclose 兼容别名 | 看 Windows 编译是否通过、测试结果如何 |
| 2026-10-24 21:07 | M1 | Windows：根目录检查改用 std::filesystem；utime 路径转 string | 看 Windows 编译与测试结果 |
| 2026-10-24 21:12 | M1 | Windows 引擎与测试已能完整编译链接；测试运行崩溃，CI 改为逐部分运行以定位 | 根据逐部分结果修复 Windows 上崩溃/失败的测试 |
| 2026-10-24 21:18 | M1 | Windows：新增 fopenShared（UTF-8 路径 + 共享删除，允许替换打开中的 WAL），引擎内 20 处 fopen 全部替换；修复 MSVC 追加模式 ftell 返回 0 导致重开 WAL 时写坏文件头的真实问题 | 确认 Part 28/15 修复；定位 3c、18、30、31、32 崩溃 |
| 2026-10-24 21:24 | M1 | Windows replaceFile uses POSIX-semantics rename | Check Windows CI parts 28/3c/56 |
| 2026-10-24 21:30 | M1 | Part 3c portable temp path; CI surfaces FAIL lines | Fix part 56 |
| 2026-10-24 21:36 | M1 | CI also reports FAIL: lines | Fix part 56 |
| 2026-10-24 21:42 | M1 | Windows scanner normalizes root to / separators | Make engine-windows blocking |
| 2026-10-24 21:48 | M1 ✅ | Engine portable: macOS 11,979 / Linux 11,910 / Windows 11,894 tests pass, 0 fail; engine-windows now blocking | M2: cxx FFI bridge + Mac scanner into jseek-platform |
| 2026-10-25 07:57 | M2 | C ABI bridge (engine/Bridge) + jseek-engine crate + jseek CLI + parity tests | Green engine-rust CI |
| 2026-10-25 07:57 | M2 | Record D-008 (C ABI bridge) | Green engine-rust CI |
| 2026-10-25 08:04 | M2 | engine-rust surfaces failures as annotations | Fix macOS engine-rust |
| 2026-10-25 08:09 | M2 | Parity tests use separate fixture dirs | Confirm engine-rust green twice |
| 2026-10-25 08:14 | M2 ✅ | C bridge + jseek-engine + jseek CLI; parity tests and CLI smoke test green on macOS and Linux | M3: Vue UI rewrite on top of jseek-engine |
| 2026-10-25 09:01 | M3 | Vue UI (tabs, QUICK, start panel, result/content rows, color cards, keys, settings) + app backend on jseek-engine, static re2 via vcpkg | Green app builds; then tray, global shortcut, autostart |
| 2026-10-25 09:02 | M3 | Tray/menu-bar icon, global shortcut, launch at login, start hidden; CI fills new lock entries | Green app builds on all 3 targets |
| 2026-10-25 09:18 | M3 | Strip Windows verbatim path prefix for CMake | Green Windows app build |
| 2026-10-25 09:18 | M3 | Strip Windows verbatim path prefix for CMake (actual fix) | Green Windows app build |
| 2026-10-25 12:14 | M3 | Release job (workflow_dispatch), notes for v2.0.0-preview.1, app version 2.0.0 | Publish v2.0.0-preview.1 |
| 2026-10-25 17:23 | M3 | Ad-hoc sign Mac app + CI signature check | Publish v2.0.0-preview.2 |
