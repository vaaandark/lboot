# lboot

[简体中文](README-zh.md) | [English](README.md)

由 Rust 语言实现的轻量级引导加载程序。

## 编译

```console
$ cargo build --release --target x86_64-unknown-uefi
```

可执行文件在 `target/x86_64-unknown-uefi/release/lboot.efi` 。

## 使用

> 程序尚在测试阶段，建议使用 qemu 模拟运行

### 配置文件

配置文件采用 [TOML](https://toml.io/) 语法的一个子集编写，配置示例如 [lboot.toml](lboot.toml) ，**应该放在 EFI 分区下，如 `/boot/lboot.toml`** 。


> 不支持注释和转义字符等语法

```toml
[[entry]]
name = "Arch Linux"
vmlinux = '\vmlinuz-linux'
param = 'initrd=\initramfs-linux.img root=UUID=48b884eb-2b80-xxxx-xxxx-xxxxxxxxxxxx rw  loglevel=3 quiet'
```

- `name` 该引导项的名称，可以为空。
- `vmlinux` 用于引导的内核可执行文件的路径，应该使用绝对路径并以 `\` 作为路径分隔符。
- `param` Linux 内核的启动参数。

### qemu 仿真

> 应使用带有图形化界面的 qemu 软件包

首先编辑配置文件 [lboot.qemu.toml](lboot.qemu.toml) 并准备好 bzImage, initrd, rootfs 等用于 Linux 启动的文件。

创建用于仿真的路径，并将配置文件等放到指定位置：

```console
$ mkdir -p test/esp/efi/boot
$ cp path/to/lboot.qemu.toml test/esp/efi/boot/lboot.toml
$ cp path/to/bzImage test/esp/efi/boot/
$ cp path/to/initramfs-linux.img test/esp/efi/boot
```

qemu 将会把 `esp` 目录认作是一个 FAT 驱动器分区，并会自动启动到其中的 `bootx64.efi` 文件。

`test` 目录下的 `qemu_run.sh` 脚本能将可执行文件复制到 `esp/efi/boot/boot{x64,aa64}.efi`，并以适当的命令行参数启动 qemu 仿真。由于这个包装脚本的存在，可以直接使用 `cargo run` 在 qemu 中运行程序：

```console
$ cargo run --target x86_64-unknown-uefi  # x86_64 架构仿真
$ cargo run --target aarch64-unknown-uefi # aarch64 架构仿真 
```

### 单元测试

单元测试在 `lboot-test-runner` 目录下，同样可以直接使用 `cargo run` 开始测试：

```console
$ cd lboot-test-runner
$ ls examples # 查看测试单元
$ cargo run --target x86_64-unknown-uefi --example menu_test # 以 x86_64 仿真测试目录模块为例
```

### 安装到操作系统

假设将编译好的 `lboot.efi` 放到了 `/boot/EFI/lboot/lboot.efi`，EFI 系统分区在硬盘 /dev/sda 上：

```console
$ sudo ./lboot-install.sh /dev/sda /boot/EFI/lboot/lboot.efi
```

并将配置文件放到 `/boot/lboot.toml` 。

### 引导界面

使用类似 grub2 的菜单界面，`>` 用于指示被选中的引导项：

```text
> Linux 6.5.5@[\efi\boot\bzImage.efi] -- initrd=\efi\boot\initramfs-linux.img
  Linux 6.4.16@[\efi\boot\bzImage6.4.16.efi] -- initrd=\efi\boot\initramfs-linux.img
  Linux 6.1.55@[\efi\boot\bzImage6.1.55.efi] -- initrd=\efi\boot\initramfs-linux.img
```

`k` 或 `UP` 向上，`j` 或 `DOWN` 向下，`RETURN` 或 `RIGHT` 或 `l` 选择进入选中引导项。

如果在 3 秒内没有选择，则自动进入第一个引导项。
