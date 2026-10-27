# Lab2 实验报告：地址空间（虚拟内存）

## 一、实验目的
- 理解 SV39 分页机制与地址空间（用户空间/内核空间、页表、trampoline）
- 掌握系统调用中用户虚拟地址的处理：页表翻译、跨页数据写入
- 实现 mmap/munmap 系统调用，为用户程序提供按需映射/解除映射能力

## 二、实验内容与关键实现

### 1. sys_get_time（虚拟内存版）
- 用户传入的 TimeVal 指针是虚拟地址，可能跨页
- 使用 translated_byte_buffer 翻译成物理缓冲区分段，将 16 字节的 TimeVal 分段写入

### 2. sys_trace（虚拟内存版）
- 读/写用户内存前，通过页表翻译（PageTable::from_token + translate）找到对应 PTE
- 检查 PTE 的 U 标志（只允许访问用户可访问页面，防止访问 trampoline 等内核页）
- 读检查 readable、写检查 writable，未映射或权限不足返回 -1
- 保留系统调用计数功能（ch3 机制）

### 3. sys_mmap
- 参数校验：prot 只允许 R/W/R|W（0 或含其他位返回 -1）、start 页对齐
- 权限映射：prot 位 0→R、位 1→W，加 U 标志
- 在 MemorySet 中检查与现有区域是否重叠，不重叠则插入新映射区域

### 4. sys_munmap
- 只允许精确匹配整块已映射区域（start 页对齐、范围完全一致），否则 -1
- 解除页表映射并从 areas 中移除，之后可重新映射

## 三、测试结果
运行 `make -C os run BASE=0`：
- Test 04_1 OK! / Test 04_4 test OK! / Test 04_5 ummap OK! / Test 04_6 ummap2 OK! / Test trace_1 OK!
- ch3 测试也通过：Test trace OK! / Test sleep OK!
- ch4_mmap1/mmap2 正确触发访存异常被内核杀死

