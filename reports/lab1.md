# Lab1 实验报告：多道程序与分时多任务（sys_trace）

## 一、实验目的
- 理解多道程序与分时多任务的基本机制（任务调度、上下文切换）
- 掌握系统调用的实现流程：用户态 ecall → trap → syscall 分发 → 内核实现
- 实现 sys_trace 系统调用，支持系统调用次数统计与用户内存读写追踪

## 二、实验内容与关键实现

### 1. 系统调用计数机制
- 在 TaskControlBlock 中新增 `syscall_times: [usize; MAX_SYSCALL_NUM]` 字段
- 在 trap_handler 的 UserEnvCall 分支中，先对本次系统调用计数，再分发
- 新增 `increment_syscall_count` 和 `get_syscall_count` 两个接口

### 2. sys_trace 系统调用
sys_trace(trace_request, id, data) 支持三种请求：
- TRACE_READ (0)：读取指定地址的 1 字节并返回
- TRACE_WRITE (1)：向指定地址写入 1 字节
- TRACE_SYSCALL (2)：返回指定系统调用的累计调用次数

（关键代码片段贴这里）

## 三、测试结果
运行 `make -C os run BASE=0` 输出：
- Test sleep1 passed!
- Test trace OK!
- Test sleep OK!
