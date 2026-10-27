# Lab3 实验报告：进程管理与基于 stride 的调度

## 一、实验目的
- 理解进程控制块（TCB）、父子进程关系、僵尸进程
- 实现 fork/exec/spawn/waitpid 等进程管理系统调用
- 将 FIFO 调度改造为基于优先级的 stride 调度

## 二、实验内容与关键实现

### 1. 进程管理
- TCB（TaskControlBlockInner）新增字段：parent（Weak 引用防循环）、children、exit_code、pass/stride/priority
- fork：复制父进程地址空间（from_existed_user），子进程 fork 返回 0，加入父的 children
- exec：加载新 elf 替换地址空间（trap 上下文需在 exec 后重新获取）
- spawn：直接加载程序创建子进程（不复制地址空间），与 fork 的区别
- waitpid/wait：只回收 Zombie 子进程（未运行 -2、无子进程 -1），孤儿进程在父退出时转交 initproc

### 2. stride 调度
- 原理：每个任务维护 pass（已运行量）和 stride（步长）
- stride = BIG_STRIDE / priority，priority 越高 stride 越小
- 调度器每次选择 pass 最小的任务，选中后 pass += stride
- 实现：TaskManager 的 fetch 从 FIFO pop 改为遍历选最小 pass，并在取出的同时累加 pass
- set_priority：优先级 ≥2 合法（返回设置值），0/1/负数返回 -1

### 3. 补充的虚拟内存系统调用
- sys_get_time 通过 translated_byte_buffer 分页写入 TimeVal
- sys_mmap/sys_munmap：mmap_area 检查重叠（floor/ceil 对齐），munmap_area 精确匹配
- sys_trace 保留：页表翻译 + U 权限检查 + 系统调用计数

## 三、测试结果
- ch5 Usertests passed!：spawn0/spawn1/set_priority/stride 全部通过
- stride 输出 ratio 基本恒定（约 390 万），CPU 按优先级成比例分配
- ch4 的 mmap/unmap/trace 测试在 ch5 下继续通过

