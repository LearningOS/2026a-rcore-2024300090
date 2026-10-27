# 实验五（ch8）：线程与死锁检测

## 实验目标
- 理解多线程与并发同步原语（互斥锁、信号量、条件变量）的实现
- 实现死锁检测机制：识别线程间循环等待资源，并通过返回错误码终止死锁线程
- 补齐缺失系统调用（sys_get_time），保证用户态 sleep 等时间相关功能的正确性

## 实现内容
1. 同步原语扩展：
   - `Mutex` trait 的 `lock()` 返回 `bool`（`false` 表示检测到死锁）；`MutexBlocking` 增加持有者（holder）追踪，解锁时把持有权移交给被唤醒线程
   - `Semaphore` 增加 `holders()`（持有者）与 `waiters()`（等待者）查询接口，down 成功时登记当前线程，up 时移除释放者
2. 全局死锁检测开关：`ENABLE_DEADLOCK_DETECT`（`UPSafeCell<bool>`），由 `sys_enable_deadlock_detect` 系统调用控制
3. 系统调用：
   - `sys_enable_deadlock_detect`：开启/关闭检测
   - `sys_mutex_lock` / `sys_semaphore_down`：检测命中时返回 `-0xdead`
   - `sys_get_time`：实现 gettimeofday 语义（此前未实现，返回 -1，导致用户态 `sleep` 忙等永远无法结束）
4. 死锁检测算法 `sem_deadlock`：从当前线程等待的信号量出发，DFS 遍历"持有-等待"图，寻找循环等待

## 关键设计
- 持有者记录：信号量 down 成功时把当前线程 tid 加入持有者列表，up 时移除释放者；wait_queue 中的线程作为等待者
- 环检测规则：
  - 主线程（tid 0）持有的资源视为必然释放，不构成死锁
  - 持有者仅等待 barrier（信号量 0）时视为"稍后会释放"，不构成等待链
  - 环闭合必须发生在不同信号量之间：多个线程竞争同一信号量是合法竞争，不是死锁
  - visited 剪枝避免重复遍历
- 返回约定：检测到死锁时系统调用返回 `-0xdead`，用户程序据此释放自身资源并退出，测试程序统计失败线程数

## 实验结果
- `make -C ci-user test CHAPTER=8`：25/25 全部通过
  - `deadlock test mutex 1 OK` / `deadlock test semaphore 1 OK` / `deadlock test semaphore 2 OK` 全部通过
  - 基础回归测试与并发测试（mpsc_sem、phil_din_mutex、threads、test_condvar、pipetest 等）全部通过

## 实验体会
- 死锁的本质是多个线程循环等待资源，检测的关键在于准确建模"持有-等待"关系，并把"真死锁环"与"合法的多线程竞争"区分开（同一信号量的竞争、等待 barrier 的暂态都不应误报）
- 调试中发现 `sys_get_time` 未实现导致用户态 `sleep` 忙等永不结束，进而使死锁测试线程无法启动——系统调用的完整性会直接影响上层应用行为，排查问题要从"编译通过"延伸到"运行行为正确"
- 通过对比评测输出逐步收窄根因（检测漏检 → 误报 → 时间服务缺失），理解了用最小可复现证据定位问题的方法