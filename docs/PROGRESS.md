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
