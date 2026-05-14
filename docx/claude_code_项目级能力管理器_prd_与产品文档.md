# Claude Code 项目级能力管理器
## PRD 与产品文档 v0.1

> 暂定产品名：**Capability Repo Manager**
>
> 面向本地 Git 仓库的 Claude Code 项目能力管理、迁移、复现与诊断工具。

---

# Part A. PRD 文档

## 1. 文档信息

| 字段 | 内容 |
|---|---|
| 文档版本 | v0.1 |
| 产品阶段 | 概念收束 / MVP 定义 |
| 目标读者 | 产品、设计、研发、后续 Spec 编写者、TDD 实施者 |
| 平台优先级 | Linux 优先，后续支持 macOS / Windows |
| 推荐技术方向 | Tauri 2 + Svelte + TypeScript + Rust + SQLite |

---

## 2. 产品概述

### 2.1 一句话定义

**Capability Repo Manager 是一个以本地 Git 仓库为中心的 Claude Code 项目能力管理工具，帮助开发者统一发现、理解、迁移、复现和校验项目级 Skills、MCP、Hooks、Rules、Agents 与相关配置。**

### 2.2 核心价值

当前 Claude Code 生态中的能力对象逐渐增多：

- Skills
- MCP Servers
- Hooks
- Rules
- Agents
- Project settings
- Plugins
- 项目知识文件与说明文件

它们分散在项目目录、全局目录、用户配置、本地私有配置之间，造成：

1. **不知道某个项目到底配置了什么能力**；
2. **难以把 A 项目上的能力复现到 B 项目**；
3. **难以追踪项目能力配置是否发生漂移**；
4. **团队难以共享一套稳定的 Claude Code 工作流**；
5. **Git 项目与 AI 能力配置之间缺少统一视图**。

本产品希望把这些能力统一建模，并围绕“仓库级能力管理”形成完整闭环：

> 仓库发现 → 能力盘点 → 能力打包 → 迁移部署 → 校验诊断 → 漂移追踪

---

## 3. 背景与问题定义

### 3.1 用户现状

典型开发者拥有多个本地 Git 项目：

- 工作项目；
- 个人项目；
- 开源项目；
- 试验性仓库；
- 多语言或 monorepo 项目。

他们可能为不同仓库配置了不同的：

- coding skill；
- code review skill；
- release skill；
- GitHub / database / browser / docs MCP；
- pre-tool hook；
- post-edit hook；
- 项目专属规则；
- 团队统一的 Claude 行为约束。

随着配置增加，管理难度急剧上升。

### 3.2 核心痛点

#### 痛点 1：看不清

- 某个项目到底有哪些 Claude 资产？
- 哪些是项目级？哪些是全局覆盖？
- 当前实际生效的是哪一份配置？
- 某个 skill 是项目独有还是复制来的？

#### 痛点 2：迁移慢

- 想把 A 项目的 skills 迁移到 B 项目；
- 想复制一套 hooks + MCP + rules；
- 需要手工找文件、复制目录、改路径、补变量；
- 容易遗漏依赖。

#### 痛点 3：复现差

- “我在这个仓库上 Claude 很好用，为什么另一个仓库复现不了？”
- 缺少 pack、manifest、lock、version 的概念；
- 迁移后没有 doctor 校验。

#### 痛点 4：团队协作难

- 团队希望共享一套 Claude 工作流；
- 但不同成员本地配置不一致；
- project scope 与 local scope 混杂；
- 没有清晰的 baseline 与 drift diff。

#### 痛点 5：仓库与 AI 配置割裂

- 多仓库用户希望先看到项目目录，而不是先看孤立配置；
- Git 是项目最稳定的边界；
- 但当前缺乏“Repo → Claude Capability”的总控工具。

---

## 4. 产品目标

### 4.1 总目标

构建一个**仓库中心化的 Claude Code 能力管理系统**，让用户可以：

1. 自动发现本机 Git 仓库；
2. 盘点每个仓库的 Claude 项目资产；
3. 打包并迁移能力配置；
4. 诊断配置缺失、冲突、不可复现等问题；
5. 持续追踪能力配置与标准模板之间的差异。

### 4.2 MVP 目标

MVP 必须解决以下 5 件事：

1. **扫描 Git 仓库**；
2. **解析仓库中的 Claude 相关配置**；
3. **展示项目能力清单**；
4. **导出 / 导入 Capability Pack**；
5. **迁移前后支持 diff、冲突提示、doctor 校验**。

### 4.3 成功标准

MVP 上线后，用户应当能够在一个典型场景下完成：

> 选择本地两个仓库 → 查看 A 仓库的 Claude 能力 → 导出能力包 → 应用到 B 仓库 → 处理冲突 → 完成校验 → 确认 B 仓库能力复现。

---

## 5. 非目标

第一阶段明确不做：

- 完整 Git 客户端；
- Git commit / branch 管理套件；
- 云端团队协作平台；
- SaaS 同步服务；
- 社区插件市场；
- 在线模板商城；
- 自动执行未知 hooks；
- 自动联网拉取任意远程配置；
- 全量 IDE 替代。

---

## 6. 目标用户

### 6.1 Persona A：多仓库个人开发者

#### 特征
- Linux 用户；
- 本地项目多；
- 已经在 Claude Code 中使用 skills / MCP / hooks；
- 经常复制配置。

#### 诉求
- 快速知道哪些项目配置完善；
- 复用成功项目上的能力；
- 减少手工复制；
- 快速排查“为何另一个项目没生效”。

---

### 6.2 Persona B：技术负责人 / Team Lead

#### 特征
- 希望团队有统一 Claude 工作流；
- 有多个 repo；
- 关注规范落地与复现。

#### 诉求
- 定义团队 blueprint；
- 检查 repo 是否偏离基线；
- 提供一键能力模板；
- 降低团队接入成本。

---

### 6.3 Persona C：平台 / AI 工具维护者

#### 特征
- 负责团队 AI 工具链；
- 关注安全、可控、可审计；
- 需要批量查看本地仓库状态。

#### 诉求
- 管理可信 capability pack；
- 检查 hooks 与 MCP 风险；
- 形成诊断报告；
- 支持迁移前 review。

---

## 7. 核心用户旅程

### 7.1 旅程 1：首次扫描本地仓库

1. 用户打开应用；
2. 选择扫描目录，例如：
   - `~/code`
   - `~/work`
3. 工具递归发现 Git repo；
4. 显示仓库列表；
5. 用户进入某仓库详情页；
6. 查看 Claude Capability Inventory。

---

### 7.2 旅程 2：从 A 项目导出能力包

1. 用户打开仓库 A；
2. 点击“导出 Capability Pack”；
3. 勾选导出内容：
   - skills；
   - hooks；
   - MCP；
   - rules；
   - agents；
4. 设置 pack 名称与说明；
5. 系统生成 pack；
6. Pack 被保存在本地 Pack Library。

---

### 7.3 旅程 3：导入能力包到 B 项目

1. 用户打开仓库 B；
2. 点击“应用 Capability Pack”；
3. 选择 pack；
4. 系统计算迁移计划：
   - 新增哪些文件；
   - 覆盖哪些文件；
   - 哪些冲突；
   - 哪些环境变量缺失；
5. 用户选择策略：
   - 跳过；
   - 覆盖；
   - 合并；
   - 重命名；
6. 执行迁移；
7. 系统运行 doctor；
8. 输出迁移结果报告。

---

### 7.4 旅程 4：检测项目是否偏离团队模板

1. 用户进入仓库 C；
2. 选择一个 baseline pack；
3. 点击“比较与模板差异”；
4. 系统显示：
   - 缺少项；
   - 多出项；
   - 修改项；
   - 版本不一致项；
5. 用户选择同步或保留差异。

---

## 8. 产品功能范围

# 8.1 模块一：Repo Catalog

## 8.1.1 功能概述

Repo Catalog 用于发现、索引并展示本机 Git 仓库，是产品的一级入口。

## 8.1.2 功能列表

### F1.1 扫描目录
- 支持手动添加扫描根目录；
- 支持多个根目录；
- 支持递归扫描；
- 支持扫描深度限制；
- 支持忽略目录。

### F1.2 仓库发现
- 识别 `.git` 目录；
- 识别 worktree 或特殊 Git 结构；
- 记录 repo root；
- 去重重复发现的路径。

### F1.3 仓库元数据
记录：
- repo id；
- absolute path；
- display name；
- remote URL；
- current branch；
- HEAD commit；
- dirty state；
- last indexed at；
- detected Claude capabilities count。

### F1.4 列表操作
- 搜索 repo；
- 按名称筛选；
- 按标签筛选；
- 按最近扫描排序；
- 按 dirty state 排序；
- 按 Claude 配置完整度排序。

### F1.5 手动刷新
- 单仓库刷新；
- 全局刷新；
- 对扫描结果增量更新。

## 8.1.3 MVP 优先级
- 必须有：F1.1 / F1.2 / F1.3 / F1.4 基础筛选 / F1.5 单仓库刷新。

---

# 8.2 模块二：Capability Inventory

## 8.2.1 功能概述

对每个仓库解析 Claude Code 项目资产，并展示能力结构与最终生效状态。

## 8.2.2 可识别对象

### Resource Types
- Skill
- MCP Server
- Hook
- Rule
- Agent
- Command
- Plugin Declaration
- CLAUDE.md / 项目上下文文档
- Settings Fragments

## 8.2.3 功能列表

### F2.1 资产识别
- 探测相关目录与配置文件；
- 解析基础元数据；
- 统计数量；
- 标识是否属于版本控制范围。

### F2.2 Scope 区分
- project scope；
- local scope；
- user-level inherited config；
- effective state。

### F2.3 资源详情
对单一资源展示：
- 名称；
- 类型；
- 文件位置；
- 来源 scope；
- 是否纳入 Git；
- 是否有潜在冲突；
- 依赖项；
- 摘要内容。

### F2.4 Effective Config View
- 展示当前仓库最终生效视图；
- 标记覆盖关系；
- 标记冲突关系；
- 标记本地私有配置与共享配置。

### F2.5 Compare View
- 比较两个仓库的能力差异；
- 比较当前仓库与 pack 的差异；
- 比较迁移前后差异。

## 8.2.4 MVP 优先级
- 必须有：F2.1 / F2.3 / 简化版 F2.5。
- 重要但可后置：完整 effective state 解析。

---

# 8.3 模块三：Capability Pack

## 8.3.1 定义

Capability Pack 是一组可迁移、可复现、可审计的 Claude 项目能力集合。

## 8.3.2 Pack 包含内容

- manifest；
- metadata；
- selected resources；
- dependencies；
- env placeholders；
- migration notes；
- validation rules；
- optional lock information。

## 8.3.3 Pack 类型

### Project Pack
从某个仓库直接导出。

### Blueprint Pack
作为可复用模板维护。

### Team Baseline Pack
团队标准配置基线。

## 8.3.4 功能列表

### F3.1 导出 Pack
- 从某 repo 创建 pack；
- 支持全量导出；
- 支持选定资源导出；
- 支持添加说明。

### F3.2 Pack Library
- 本地 pack 列表；
- 查看详情；
- 查看引用来源；
- 查看版本；
- 删除 / 复制。

### F3.3 Pack 结构校验
- manifest schema 校验；
- 文件完整性校验；
- 资源引用校验。

### F3.4 版本化
- pack version；
- created from repo；
- source commit；
- changelog。

## 8.3.5 MVP 优先级
- 必须有：F3.1 / F3.2 基础列表 / F3.3。
- 可后置：复杂版本管理与 changelog UI。

---

# 8.4 模块四：Migration Engine

## 8.4.1 功能概述

负责将 Capability Pack 应用到目标仓库，并在执行前生成可审计的迁移计划。

## 8.4.2 迁移对象

- pack → repo；
- repo → repo；
- repo → pack；
- 后续可扩展 pack → multiple repos。

## 8.4.3 功能列表

### F4.1 Dry-run 计划
计划内容：
- 新增资源；
- 已存在资源；
- 冲突资源；
- 潜在覆盖；
- 目标路径；
- 依赖缺失；
- env 缺失。

### F4.2 冲突解决策略
- skip；
- overwrite；
- rename；
- merge；
- manual resolve placeholder。

### F4.3 执行迁移
- 应用策略；
- 写入文件；
- 记录历史；
- 输出结果。

### F4.4 回滚机制
- 执行前自动 snapshot；
- 支持单次撤销；
- 支持恢复到迁移前状态。

### F4.5 Migration Report
- 已新增；
- 已覆盖；
- 已跳过；
- 失败项；
- 后续建议。

## 8.4.4 MVP 优先级
- 必须有：F4.1 / F4.2 skip-overwrite-rename / F4.3 / F4.5。
- 强烈建议有：基础 snapshot rollback。

---

# 8.5 模块五：Doctor & Drift

## 8.5.1 功能概述

用于判断某个 repo 的能力配置是否完整、健康、可复现。

## 8.5.2 Doctor 检查项

### D1. 结构检查
- skill 目录是否完整；
- 必要文件是否缺失；
- manifest 是否可读；
- settings 是否可解析。

### D2. 依赖检查
- MCP 是否定义完整；
- hook 引用的 script 是否存在；
- 外部命令是否可能缺失；
- env placeholder 是否未替换。

### D3. 风险提示
- hooks 是否包含可执行命令；
- 是否依赖外部联网；
- 是否存在敏感变量模板；
- 是否存在目标路径外写入风险提示。

### D4. Drift 检查
- repo vs pack；
- repo vs baseline；
- manifest version mismatch；
- missing / modified / extra resource。

## 8.5.3 输出形态
- 健康度评分；
- 问题分级：
  - Critical
  - Warning
  - Info
- 修复建议；
- 快捷跳转到对应资源。

## 8.5.4 MVP 优先级
- 必须有：D1 / D2 基础能力 / D4 简化版 diff。
- 可后置：细粒度风险模型。

---

## 9. 信息架构

### 9.1 一级导航建议

1. **Repositories**
2. **Packs**
3. **Compare**
4. **Diagnostics**
5. **Settings**

---

### 9.2 页面结构

#### 9.2.1 Repositories 页面
- 顶部：搜索栏 / 扫描按钮；
- 左侧：标签与过滤器；
- 主体：repo 列表；
- 卡片字段：
  - repo 名称；
  - path；
  - branch；
  - dirty state；
  - Claude 资源数量；
  - doctor 状态。

#### 9.2.2 Repo Detail 页面
Tabs：
- Overview
- Capabilities
- Pack Export
- Compare
- Doctor
- Activity / History

#### 9.2.3 Packs 页面
- Pack Library；
- Pack Detail；
- Apply to Repo；
- Version Info。

#### 9.2.4 Compare 页面
- Repo vs Repo；
- Repo vs Pack；
- Pack vs Pack。

#### 9.2.5 Diagnostics 页面
- 全局异常聚合；
- 某类错误筛选；
- 最近迁移失败记录。

---

## 10. 核心对象模型

# 10.1 Domain Entities

## Repository
```yaml
id: string
name: string
path: string
remoteUrl: string | null
currentBranch: string | null
headCommit: string | null
dirtyState: clean | modified | unknown
lastIndexedAt: datetime
capabilitySummary:
  skillCount: number
  mcpCount: number
  hookCount: number
  ruleCount: number
  agentCount: number
```

## CapabilityResource
```yaml
id: string
repoId: string | null
packId: string | null
type: skill | mcp | hook | rule | agent | command | plugin | settings | contextDoc
name: string
sourcePath: string | null
scope: project | local | user | inherited | unknown
trackedByGit: boolean
hash: string | null
summary: string | null
dependencies: ResourceDependency[]
metadata: object
```

## ResourceDependency
```yaml
type: file | command | env | mcp | skill | hook | plugin
name: string
required: boolean
status: satisfied | missing | unknown
```

## CapabilityPack
```yaml
id: string
name: string
version: string
description: string | null
createdAt: datetime
createdFromRepoId: string | null
sourceCommit: string | null
resources: CapabilityResource[]
manifestPath: string
validationRules: ValidationRule[]
```

## MigrationPlan
```yaml
id: string
sourceType: repo | pack
sourceId: string
targetRepoId: string
items: MigrationPlanItem[]
conflicts: MigrationConflict[]
missingDependencies: ResourceDependency[]
createdAt: datetime
```

## MigrationPlanItem
```yaml
resourceId: string
action: add | overwrite | skip | rename | merge | unresolved
sourcePath: string | null
targetPath: string | null
status: pending | applied | failed
```

## MigrationConflict
```yaml
resourceName: string
resourceType: string
reason: same_name | same_path | incompatible_schema | ambiguous_merge
recommendedActions: string[]
```

## DoctorReport
```yaml
id: string
repoId: string
createdAt: datetime
score: number
issues: DoctorIssue[]
```

## DoctorIssue
```yaml
severity: critical | warning | info
code: string
message: string
resourceRef: string | null
recommendation: string | null
```

---

## 11. Capability Pack 建议结构

```text
my-pack/
├─ pack.manifest.json
├─ README.md
├─ resources/
│  ├─ skills/
│  ├─ hooks/
│  ├─ agents/
│  ├─ rules/
│  └─ commands/
├─ mcp/
│  └─ project.mcp.template.json
├─ settings/
│  └─ settings.template.json
└─ validation/
   └─ doctor.rules.json
```

### 示例 Manifest

```json
{
  "schemaVersion": "1.0",
  "name": "frontend-team-baseline",
  "version": "0.1.0",
  "description": "Frontend project Claude Code baseline",
  "source": {
    "repo": "local:/home/user/code/app-a",
    "commit": "abc123"
  },
  "resources": [
    {
      "type": "skill",
      "name": "ui-code-review",
      "source": "resources/skills/ui-code-review"
    },
    {
      "type": "hook",
      "name": "prevent-secret-write",
      "source": "resources/hooks/prevent-secret-write"
    }
  ],
  "env": [
    {
      "name": "GITHUB_TOKEN",
      "required": false,
      "description": "Optional for GitHub MCP"
    }
  ],
  "validation": {
    "rules": [
      "skill-structure-valid",
      "hook-target-exists",
      "mcp-config-parseable"
    ]
  }
}
```

---

## 12. 权限与安全设计

### 12.1 原则
- 扫描与执行分离；
- 默认只读；
- 对未知 hook 不自动执行；
- 对 MCP 连接测试明确标记可能产生外部访问；
- 对迁移写操作必须可预览。

### 12.2 风险动作分类

| 动作 | 风险等级 | 默认策略 |
|---|---:|---|
| 扫描 repo | 低 | 允许 |
| 读取配置 | 低 | 允许 |
| 生成 diff | 低 | 允许 |
| 写入 pack | 中 | 用户触发 |
| 应用 pack | 中 | 用户触发并预览 |
| 执行 hook 校验脚本 | 高 | 默认不执行，仅静态检查 |
| 测试 MCP 连接 | 中 | 用户触发 |

---

## 13. 核心需求优先级

### P0 - MVP 必须完成
- 目录扫描 Git repo；
- 仓库索引列表；
- 读取 repo metadata；
- 识别基础 Claude 配置资源；
- Repo Detail 页；
- 导出 Capability Pack；
- 选择 Pack 应用到另一个 Repo；
- Dry-run plan；
- 基础冲突处理；
- 执行迁移；
- 迁移报告；
- Doctor 基础检查。

### P1 - 强化体验
- Repo vs Repo compare；
- Repo vs Pack drift compare；
- Snapshot rollback；
- 标签与分组；
- 搜索增强；
- 文件监听自动刷新；
- pack version history；
- 全局 Diagnostics 页面。

### P2 - 后续扩展
- 团队 blueprint 管理；
- 批量同步多个 repo；
- 分享 pack；
- 远程模板仓库；
- 跨客户端统一配置；
- 插件生态对接。

---

## 14. 关键非功能需求

### 14.1 性能
- 500 个 repo 内应可正常使用；
- 扫描任务支持取消；
- 解析结果支持缓存；
- 列表切换要即时响应；
- 单 repo detail 页面打开不应依赖重新全量扫描。

### 14.2 稳定性
- 配置文件解析失败不得导致应用崩溃；
- 单个 repo 异常不影响全局扫描；
- 所有写操作必须有失败回滚路径。

### 14.3 跨平台
- Linux 优先完整可用；
- 文件路径抽象必须跨平台；
- Git CLI 执行层避免硬编码 shell 差异；
- 后续兼容 macOS / Windows。

### 14.4 可测试性
- Domain 与 I/O 分离；
- pack engine 可纯函数化测试；
- parser 使用 fixture 测试；
- migration 使用 golden file 测试；
- doctor 使用规则输入输出测试。

---

## 15. 验收标准

# 15.1 Repo 扫描

### AC-REPO-001
给定一个根目录，系统能发现该目录下的多个 Git repo，并去重展示。

### AC-REPO-002
每个 repo 至少展示：名称、路径、当前分支、dirty state。

### AC-REPO-003
当某个目录无访问权限时，系统跳过并记录告警，不崩溃。

---

# 15.2 Capability Inventory

### AC-CAP-001
系统可识别一个 repo 中存在的 skills、hooks、MCP、rules、agents 基础资源。

### AC-CAP-002
用户进入 Repo Detail 后，可看到资源类型分组与数量。

### AC-CAP-003
解析失败的资源会标记错误，而不是静默丢失。

---

# 15.3 Pack 导出

### AC-PACK-001
用户可从 repo 选择至少一种资源导出 pack。

### AC-PACK-002
导出后的 pack 包含 manifest 文件。

### AC-PACK-003
缺失文件或无效资源会在导出前提示。

---

# 15.4 Pack 应用

### AC-MIG-001
用户应用 pack 前，系统必须生成 dry-run 迁移计划。

### AC-MIG-002
如果目标 repo 存在同名资源，系统必须显示冲突。

### AC-MIG-003
用户选择策略后，系统能够执行迁移并生成 report。

### AC-MIG-004
执行失败时，不得留下未解释的半成品状态。

---

# 15.5 Doctor

### AC-DOC-001
系统可检测 skill 结构不完整。

### AC-DOC-002
系统可检测 hook 引用脚本不存在。

### AC-DOC-003
系统可检测 pack 所需 env placeholder 未满足。

---

## 16. 建议的 TDD 切分

### 16.1 第 1 批：Repo Scanner
- `find_git_repositories(root)`
- `read_git_metadata(repo)`
- `dedupe_repo_paths(...)`
- `handle_permission_denied(...)`

### 16.2 第 2 批：Capability Parser
- `detect_resource_roots(repo)`
- `parse_skill_dir(...)`
- `parse_mcp_config(...)`
- `parse_hook_config(...)`
- `build_capability_inventory(...)`

### 16.3 第 3 批：Pack Engine
- `create_pack_manifest(...)`
- `copy_selected_resources(...)`
- `validate_pack_schema(...)`
- `calculate_pack_hashes(...)`

### 16.4 第 4 批：Migration Engine
- `build_migration_plan(...)`
- `detect_path_conflicts(...)`
- `apply_conflict_strategy(...)`
- `execute_migration(...)`
- `generate_migration_report(...)`

### 16.5 第 5 批：Doctor
- `check_skill_structure(...)`
- `check_hook_target_exists(...)`
- `check_missing_env(...)`
- `check_pack_drift(...)`

---

## 17. MVP 开发里程碑建议

### Milestone 1：Repository Foundation
- App skeleton；
- 扫描目录；
- Repo 列表；
- Git metadata；
- SQLite 持久化。

### Milestone 2：Capability Inventory
- 解析基础配置；
- Repo Detail；
- 资源列表；
- 错误状态显示。

### Milestone 3：Pack Export
- 选择资源；
- 导出 pack；
- Pack Library；
- Manifest schema。

### Milestone 4：Migration
- Dry-run；
- 冲突处理；
- 应用 pack；
- Report。

### Milestone 5：Doctor
- 基础诊断；
- Migration 后自动诊断；
- 显示问题与建议。

---

# Part B. 产品文档

## 18. 产品愿景

在 Agentic Coding 趋势下，一个项目不再只有代码、依赖与配置文件，还会逐渐拥有属于自己的 AI 工作方式：

- 它如何理解仓库；
- 它有哪些能力；
- 它能连接哪些外部工具；
- 它在什么时机触发自动化；
- 它遵循哪些团队规则。

Capability Repo Manager 希望成为这类**项目级 AI 能力资产**的管理入口。

### 长期愿景

> 让每个代码仓库的 AI 工作流像依赖管理一样可见、可复制、可审计、可升级。

---

## 19. 产品原则

### 19.1 Repo-first
所有能力围绕 Git 仓库组织，而不是围绕孤立的配置文件组织。

### 19.2 Visibility before automation
先让用户看清楚，再做自动化迁移。

### 19.3 Dry-run by default
所有写入前必须有可审阅的计划。

### 19.4 Reproducibility over convenience
比起“临时复制成功”，更重视“以后还能复现”。

### 19.5 Safe local tooling
默认只读，执行类操作显式触发。

---

## 20. 产品命名候选

### 候选 1：Capability Repo Manager
优点：表达完整，技术感强。

### 候选 2：Claude Project Forge
优点：有产品感，暗示“装配能力”。

### 候选 3：RepoSkill
优点：短，易记，但范围偏窄。

### 候选 4：Mosaic for Claude Code
优点：暗示能力拼装，但语义不够直接。

### 当前建议
内部开发阶段可先使用：

> **Capability Repo Manager**

---

## 21. 主要场景描述

### 21.1 “我在 A 项目上调得很好，复制到 B 项目”

这是最核心场景。

用户不想重新：
- 建 skill；
- 配 MCP；
- 复制 hook；
- 改路径；
- 手工确认是否缺依赖。

工具应该让这个过程变成：

> Export → Preview → Apply → Doctor

---

### 21.2 “我想知道哪个项目的 Claude 配置最成熟”

Repo 列表不只显示 Git 状态，还显示：
- skill 数量；
- MCP 数量；
- doctor 健康度；
- 是否存在 pack baseline。

---

### 21.3 “团队有一个基线，我想看哪些 repo 跑偏了”

团队定义一个 Team Baseline Pack。

工具支持：
- 与基线对比；
- 找出缺失与修改；
- 一键修复或保留差异。

---

### 21.4 “我想给新项目一键套用工作流”

新 repo 初始化后：
- 选择 blueprint；
- 应用 pack；
- 补全 env；
- 完成 doctor；
- 即可形成基础 AI 工作环境。

---

## 22. 产品结构建议

### 22.1 首页定位
首页即 Repositories Dashboard。

原因：
- 用户首先关心项目；
- 配置管理是围绕项目展开；
- Git 仓库是最自然的入口。

### 22.2 Dashboard 关键卡片
- Total repos；
- Repos with Claude config；
- Recent migrations；
- Repos with doctor warnings；
- Packs available。

---

## 23. 详细页面说明

# 23.1 Repository Dashboard

### 目标
让用户快速找到项目，并判断当前 AI 能力状态。

### 核心组件
- Global search；
- Scan folders；
- Repo table / cards；
- Filter chips；
- Sorting controls。

### 建议字段
| 字段 | 说明 |
|---|---|
| Name | 项目名 |
| Path | 本地路径 |
| Branch | 当前分支 |
| Status | Git clean / dirty |
| Capabilities | S / M / H / R / A 数量 |
| Health | doctor 状态 |
| Updated | 最近扫描时间 |

---

# 23.2 Repo Detail

### 顶部摘要
- Repo name；
- absolute path；
- remote URL；
- current branch；
- last commit；
- dirty state；
- last scanned at。

### Tabs
1. Overview
2. Capabilities
3. Export Pack
4. Apply Pack
5. Compare
6. Doctor
7. Activity

---

# 23.3 Capabilities Tab

### 左侧分类
- Skills
- MCP
- Hooks
- Rules
- Agents
- Commands
- Plugins
- Settings

### 右侧列表
每个资源显示：
- 名称；
- 类型；
- 来源；
- 是否 tracked；
- dependency status；
- risk level；
- 详情入口。

---

# 23.4 Export Pack

### 流程
1. 选择资源；
2. 输入 pack 名称；
3. 输入版本号；
4. 输入说明；
5. 预览 pack 内容；
6. 导出。

### 设计重点
- 用户要清楚导出了哪些东西；
- 必须提示是否包含 local-only 配置；
- 必须提示是否存在敏感变量引用。

---

# 23.5 Apply Pack

### 流程
1. 选择 pack；
2. 选择目标 repo；
3. 生成 plan；
4. 解决冲突；
5. 应用；
6. 查看报告；
7. 查看 doctor。

### 迁移计划展示
- Resource name；
- Resource type；
- Planned action；
- Conflict state；
- User strategy。

---

# 23.6 Compare

### 比较模式
- Repo vs Repo；
- Repo vs Pack；
- Repo vs Baseline。

### 比较结果
- Missing；
- Extra；
- Modified；
- Same；
- Unknown。

---

# 23.7 Doctor

### Doctor 结果摘要
- Score；
- Critical issues；
- Warnings；
- Suggestions。

### 问题列表
- issue code；
- message；
- impacted resource；
- recommendation；
- jump action。

---

## 24. 交互策略

### 24.1 空状态

#### 没有扫描目录
提示：
> 添加一个代码目录，开始发现本地 Git 项目。

#### 没有 Pack
提示：
> 从一个配置成熟的项目导出第一个 Capability Pack。

#### 没有问题
提示：
> 当前项目通过基础诊断。

---

### 24.2 错误状态

- 配置文件解析失败；
- 目标路径不可写；
- Pack manifest 无效；
- Git metadata 无法读取；
- 迁移中断。

错误信息必须：
- 明确；
- 面向行动；
- 可定位。

---

## 25. 技术架构文档摘要

### 25.1 建议技术栈

| 层级 | 技术 |
|---|---|
| Desktop Shell | Tauri 2 |
| Frontend | Svelte + TypeScript + Vite |
| Backend Core | Rust |
| Local DB | SQLite |
| Git Access | v1 调用系统 Git CLI |
| File Watch | Rust notify |
| Schema | JSON Schema |

---

### 25.2 模块划分

```text
crates/
├─ domain/
├─ repo-scanner/
├─ git-service/
├─ claude-parser/
├─ pack-engine/
├─ migration-engine/
├─ doctor-engine/
├─ storage/
└─ tauri-bridge/
```

---

## 26. API / Command 建议

### Tauri Commands

```text
scan_repositories(paths)
list_repositories(filter)
refresh_repository(repo_id)
get_repository_detail(repo_id)
get_capability_inventory(repo_id)
export_capability_pack(repo_id, selection, metadata)
list_packs()
get_pack_detail(pack_id)
build_migration_plan(pack_id, target_repo_id)
apply_migration_plan(plan_id, strategies)
run_doctor(repo_id)
compare_repo_with_pack(repo_id, pack_id)
```

---

## 27. 数据库表建议

### repositories
- id
- path
- name
- remote_url
- branch
- head_commit
- dirty_state
- last_indexed_at

### capability_resources
- id
- repo_id
- type
- name
- source_path
- scope
- tracked_by_git
- content_hash
- metadata_json

### packs
- id
- name
- version
- description
- manifest_path
- source_repo_id
- source_commit
- created_at

### migration_runs
- id
- source_type
- source_id
- target_repo_id
- status
- report_json
- created_at

### doctor_reports
- id
- repo_id
- score
- issues_json
- created_at

---

## 28. 测试策略文档

### 28.1 单元测试
- Repo scanning；
- Git metadata parser；
- Capability parser；
- Pack manifest builder；
- Migration planning；
- Doctor rules。

### 28.2 集成测试
- 用 fixture repo 测试完整导出；
- 用 fixture pack 测试完整导入；
- 用冲突 repo 测试迁移 plan；
- 用错误配置 repo 测试 doctor。

### 28.3 Snapshot / Golden Tests
- Capability inventory 输出；
- Migration plan JSON；
- Doctor report JSON；
- Pack manifest。

### 28.4 E2E 测试
- 扫描目录；
- 打开 repo；
- 导出 pack；
- 导入另一个 repo；
- 查看迁移报告。

---

## 29. Spec 驱动开发建议

### 29.1 Spec 书写顺序

1. Repository Discovery Spec
2. Repository Metadata Spec
3. Capability Detection Spec
4. Capability Pack Spec
5. Migration Plan Spec
6. Migration Execution Spec
7. Doctor Rules Spec
8. Drift Compare Spec

### 29.2 每份 Spec 建议结构

- Purpose
- Definitions
- Inputs
- Outputs
- State Transitions
- Business Rules
- Error Cases
- Acceptance Criteria
- Test Matrix

---

## 30. 迭代路线图

### v0.1 - Foundation
- Linux 桌面 App；
- Git repo 扫描；
- Repo 列表；
- 基础 capability inventory。

### v0.2 - Pack
- Pack 导出；
- Pack Library；
- Manifest 校验。

### v0.3 - Migration
- Dry-run；
- 应用 pack；
- 冲突处理；
- 迁移报告。

### v0.4 - Doctor
- 结构诊断；
- 依赖检查；
- 初步 drift。

### v0.5 - Productization
- 文件监听；
- Snapshot rollback；
- Compare 增强；
- Linux 发布包。

---

## 31. 最终产品边界总结

### 本产品不是
- Git 客户端；
- Claude Code 的替代品；
- 插件市场；
- 云协作平台。

### 本产品是
- **本地 Git 仓库索引器**；
- **Claude 项目能力盘点器**；
- **Capability Pack 管理器**；
- **项目能力迁移引擎**；
- **复现与漂移诊断工具**。

---

## 32. 一句话版本

> 让 Claude Code