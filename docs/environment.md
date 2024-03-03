# Lab 0: 环境配置

## 实验环境配置

我们推荐你使用 Linux 系统完成实验. Windows 系统用户可以考虑 [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) 或者虚拟机.
为了避免不必要的问题，推荐使用 Ubuntu 22.04 LTS 或者 Debian 12 (助教的环境). 当然其他发行版/系统也是可以的, 只要你能正常安装和使用相关工具.
以下的工具版本均为 Debian 12 上的版本 (其他版本也可以).

1. 基础编译器
    + gcc 10.2.1
    + bison 3.7.5
    + flex 2.6.4
    + rust (stable)
2. 交叉编译器
    + clang 14.0.6
    + llvm 14.0.6
    + lld 14.0.6
3. riscv 虚拟机
    + qemu-user-static 7.2.9

以上工具除了 rust 之外均可以用系统自带的包管理器 (比如 apt) 安装. 

### Rust 安装
由于 Accipit IR 相关工具是用 Rust 编写的, 所以需要安装 Rust (你也可以等到 Lab 3 的时候安装). 我们推荐使用[浙大源](https://mirrors.zju.edu.cn/docs/rustup/)安装 [rustup](https://rustup.rs/). 
```bash
export RUSTUP_DIST_SERVER=https://mirrors.zju.edu.cn/rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
一般默认安装 stable 版本. 执行 `rustc --version` 检查是否安装成功. 应当输出类似 `rustc 1.76.0 (07dca489a 2024-02-04)` 的信息.

你可以尝试运行 IR 的样例来检查是否安装成功. 
```bash
git clone https://git.zju.edu.cn/accsys/accipit
cd accipit
cargo run -- examples/add.acc
```
输出形如
```
...

Interepted: Ok(Integer(2))
```

### Runtime 编译
我们提供了一个 SysY 的运行时库, 你可以在 [这里](https://git.zju.edu.cn/accsys/sysy-runtime-lib) 找到它.
你可以尝试编译它来检查交叉工具链是否配置正确. 

对于 x86_64:
```bash
make test
./build/test < test/test.in
```
对于 riscv:
```bash
make NO_LIBC=1 ADD_CFLAGS="-target riscv64-unknown-linux-elf -march=rv64im -mabi=lp64" test
qemu-riscv64-static ./build/test < test/test.in
```
你应当会看到类似如下输出
```
10: 1 -1 0 0 10 -10 -12309714 1387487 12 3
50000000
-972781568
abcdefghijklmnopqrstuvwxyz
Timer: 0H-0M-12S-124900us
TOTAL: 0H-0M-12S-124900us
```

> 如果你对编译选项感兴趣, 这里是一个简单的解释: `NO_LIBC=1` 表示不使用系统的 C 标准库 (也就是 `libc`), `ADD_CFLAGS="-target riscv64-unknown-linux-elf -march=rv64im -mabi=lp64"` 表示添加交叉编译选项. 这里 `riscv64-unknown-linux-elf` 是交叉编译器的 target triple, `rv64im` 是 riscv 的指令集, `lp64` 是 riscv 的 ABI. 注意在编译你的程序的时候也许还需要添加一个选项 `-fuse-ld=lld` 来使用 `lld` 作为 linker, 否则默认的 `/usr/bin/ld` 是无法交叉编译的, 这点已经在 Makefile 中处理了.
<br><br>
关于为什么不用 `libc`, `qemu-user-static` 是一个用户态的 riscv 虚拟机, 就像你在 OS 实验中用 `qemu-system-riscv64` 运行的系统一样, 它是不带标准库的. 如果你想输入输出, 你需要手动调用 `syscall`. 这个工作在之前是由 `libc` 来帮你完成的, 你只需要调用它提供的库函数 (比如 `printf`), 我们提供的 runtime 也是这样实现的. 
<br><br>
如果你实在想用 `libc` (比如说你想要用 C++ 写 runtime) 也是可以实现的, 但是助教并没有在 Debian 12 上配出来. 你可以考虑使用 ArchLinux, 它对交叉编译工具链的支持更友好一些 (至少助教在 ArchLinux 上配出来了).
