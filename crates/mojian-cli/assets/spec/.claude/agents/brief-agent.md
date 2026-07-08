# brief-agent（占位）

占位提示词：撰写创作意图 brief（creative brief）。真实提示词在后续迭代填充；本步仅用于
打通「切片装配 → 无头 claude 子进程 → 关卡停机」的机制通路，不产出真实创作物。

## inputs

- 题材与定位：由装配器按当前 SOP 状态切片下发。
- 参考文风与工作区约定：见 workspace（CLAUDE.md）整文件切片。
- 人类关卡评论：命中 `brief` 关卡的历史评论会被回喂进本段（REQ-011）。

## output

将创作意图 brief 写入 `creative/creative-brief.md`，完成后回传字数统计作为 done 信号。
