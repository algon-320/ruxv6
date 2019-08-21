xv6.img: bootloader/mbr kernel/kernel
	dd if=/dev/zero of=xv6.img count=10000
	dd if=bootloader/mbr of=xv6.img conv=notrunc
	dd if=kernel/kernel of=xv6.img seek=1 conv=notrunc

bootloader/mbr: bootloader/src/*
	make -C bootloader mbr

kernel/kernel: kernel/src/*
	make -C kernel kernel

qemu: xv6.img fs.img
	qemu-system-i386 -drive file=xv6.img,index=0,media=disk,format=raw -drive file=fs.img,index=1,media=disk,format=raw -smp 2 -m 512

GDBPORT = $(shell expr `id -u` % 5000 + 25000)
qemu-gdb: xv6.img fs.img
	qemu-system-i386 -drive file=xv6.img,index=0,media=disk,format=raw -drive file=fs.img,index=1,media=disk,format=raw -smp 2 -m 512 -S -gdb tcp::$(GDBPORT)

clean:
	rm xv6.img ; \
	make -C bootloader clean ; \
	make -C kernel clean
