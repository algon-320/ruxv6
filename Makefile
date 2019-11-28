xv6.img: mbr_dummy kernel_dummy
	dd if=/dev/zero of=xv6.img count=10000
	dd if=bootloader/mbr of=xv6.img conv=notrunc
	dd if=kernel/kernel of=xv6.img seek=1 conv=notrunc

xv6-debug.img: mbr_dummy kernel-debug_dummy
	dd if=/dev/zero of=xv6-debug.img count=10000
	dd if=bootloader/mbr of=xv6-debug.img conv=notrunc
	dd if=kernel/kernel-debug of=xv6-debug.img seek=1 conv=notrunc


mbr_dummy:
	make -C bootloader mbr

kernel_dummy:
	make -C kernel kernel

kernel-debug_dummy:
	make -C kernel kernel-debug

qemu: xv6.img fs.img
	qemu-system-i386 -drive file=xv6.img,index=0,media=disk,format=raw -drive file=fs.img,index=1,media=disk,format=raw -smp 2 -m 512 -serial mon:stdio

qemu-debug: xv6-debug.img fs.img
	qemu-system-i386 -drive file=xv6-debug.img,index=0,media=disk,format=raw -drive file=fs.img,index=1,media=disk,format=raw -smp 2 -m 512 -serial mon:stdio

GDBPORT = $(shell expr `id -u` % 5000 + 25000)
qemu-gdb: xv6-debug.img fs.img
	qemu-system-i386 -drive file=xv6-debug.img,index=0,media=disk,format=raw -drive file=fs.img,index=1,media=disk,format=raw -smp 2 -m 512 -S -gdb tcp::$(GDBPORT)

clean:
	rm xv6.img ; \
	rm xv6-debug.img ; \
	make -C bootloader clean ; \
	make -C kernel clean
