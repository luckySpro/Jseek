# JSeek（简搜）

在 Mac 和 Windows 上毫秒级找到任何文件，并能搜索文件内容（带行号）。一套代码，同样的界面、语法和快捷键；离线可用，常驻菜单栏/托盘。

> 状态：规划中。方案见 [docs/PLAN.md](docs/PLAN.md)，决策见 [docs/DECISIONS.md](docs/DECISIONS.md)，进度见 [docs/PROGRESS.md](docs/PROGRESS.md)。
> Mac 现有版本：[JMacEverything](https://github.com/luckySpro/JMacEverything)（JSeek 的前身）。

## 开发约定

- `main` 只放稳定内容；日常开发在 `dev`。
- 每完成一个可描述的步骤运行：`scripts/checkpoint.sh "<阶段>" "<完成了什么>" "<下一步>"`，自动记录进度、提交、推送并核对远端。
- 中断后从 `docs/PROGRESS.md` 最后一行的「下一步」继续。

## 致谢

搜索引擎源自 [joshua-wu/MacEverything](https://github.com/joshua-wu/MacEverything)（MIT）；内容搜索思路参考 [noahdunnagan/fsearch](https://github.com/noahdunnagan/fsearch)。

© M&J Group · 473276@qq.com
