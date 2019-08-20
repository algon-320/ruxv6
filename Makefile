mbr: src/*.rs src/*.S i386.json ./linker.ld
	RUST_TARGET_PATH=$(shell pwd) xargo build --target i386 --release
	objcopy -O binary -j .text -j .rodata -j .signature ./target/i386/release/ruxv6 ./mbr

xv6.img: mbr kernel
	dd if=/dev/zero of=xv6.img count=10000
	dd if=mbr of=xv6.img conv=notrunc
	dd if=kernel of=xv6.img seek=1 conv=notrunc

qemu: xv6.img fs.img
	qemu-system-i386 -drive file=xv6.img,index=0,media=disk,format=raw -drive file=fs.img,index=1,media=disk,format=raw -smp 2 -m 512

clean:
	xargo clean
	rm ./bootloader ./mbr ./xv6.img
