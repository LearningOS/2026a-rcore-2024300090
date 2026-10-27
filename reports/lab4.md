# Lab4 实验报告：文件系统与硬链接

## 实验目标

在 rCore 中实现文件系统相关系统调用 `fstat`、`linkat`、`unlinkat`，并扩展 easy-fs 文件系统以支持硬链接（nlink 计数），同时补齐 ch6 模板中缺失的 `get_time`、`mmap`、`munmap`、`spawn` 系统调用。

## 实现内容

### 1. easy-fs 层扩展（easy-fs/src）

- **layout.rs**：`DiskInode` 增加 `nlink: u32` 字段，`initialize` 中初始化为 0；`create` 后文件 nlink 置 1。
- **efs.rs**：新增 `get_inode_id(block_id, block_offset)`，作为 `get_disk_inode_pos` 的逆运算，用于 fstat 返回 inode 号。
- **vfs.rs**：`Inode` 新增方法：
  - `link(name, new_name)`：在目录中为 `name` 指向的 inode 追加一个新目录项 `new_name`，并将目标 inode 的 `nlink += 1`；
  - `unlink(name)`：删除目录项（用最后一个目录项覆盖待删项并缩小目录 size），`nlink -= 1`；
  - `inode_id()` / `nlink()` / `is_dir()`：供 fstat 查询属性。

### 2. 内核文件系统层（os/src/fs）

- **mod.rs**：`File` trait 增加 `fstat(&self, st: &mut Stat)` 方法，并导出 `ROOT_INODE`。
- **inode.rs**：`OSInode` 实现 `fstat`，填充 `dev=0`、`ino=inode_id`、`mode`（文件/目录）、`nlink`。
- **stdio.rs**：`Stdin`/`Stdout` 补空实现的 `fstat`。

### 3. 系统调用层（os/src/syscall）

- **fs.rs**：
  - `sys_fstat(fd, st)`：从 fd_table 取文件，经 `translated_refmut` 写回用户 Stat；
  - `sys_linkat(old, new)`：`ROOT_INODE.link()`，成功返回 0；
  - `sys_unlinkat(name)`：`ROOT_INODE.unlink()`，成功返回 0。
- **process.rs**：
  - `sys_get_time`：`translated_byte_buffer` 分段写回 16 字节 `TimeVal`（处理跨页）；
  - `sys_mmap`/`sys_munmap`：页对齐校验 + `MapPermission` 转换，调用 memory_set 的 `mmap_area`/`munmap_area`；
  - `sys_spawn`：通过 `open_file` 从文件系统加载 ELF（ch6 无 loader），`spawn()` 新建进程并入队。

### 4. 内存与进程层

- **memory_set.rs**：`mmap_area`（新区域与已有区域重叠则拒绝；`floor()/ceil()` 计算 VPN，避免非页对齐 panic）、`munmap_area`（精确匹配页对齐范围后 unmap 并移除）。
- **task.rs**：`spawn()` —— 从 ELF 直接构建新地址空间（区别于 fork 复制父进程地址空间），父进程指向当前进程并注册为子进程，fd_table 含 stdin/stdout/stderr。
- **task/mod.rs**：`current_memory_set_mmap/munmap` 桥接。

## 关键设计说明

- **硬链接语义**：`link` 只是新增目录项指向同一 inode，数据块不复制，靠 nlink 计数；`unlink` 删除目录项并递减 nlink，数据块允许残留（评测注释明确允许）。
- **spawn 与 fork 的区别**：fork 复制父进程地址空间和 fd_table；spawn 用新 ELF 重新加载地址空间，fd_table 只保留标准三流，且不复制用户栈。
- **跨页写入**：`sys_get_time` 的目标 `TimeVal` 可能跨两个用户页，必须通过页表逐段写回，不能直接解引用。

## 实验结果

- 本地全量测试（ch6_usertest 作 initproc）：`Test file0 OK!`、`Test fstat OK!`、`Test link OK!`、`Test mass open/unlink OK!` 全部通过，`ch6 Usertests passed!`。
- ci-user 正式评测：31/31 PASS。

## 实验体会

文件系统的硬链接本质上是在"目录项"层面做文章：多个名字共享一个 inode，只有 nlink 归零才真正删除数据。ch6 让我理解了 VFS 与磁盘布局（DiskInode 块内偏移、目录项数组）的关系，以及"系统调用 → VFS → 磁盘 inode"的三层职责划分。
