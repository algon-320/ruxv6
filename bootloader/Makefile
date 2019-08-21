mbr: src/*.rs src/*.S i386.json ./bootloader.ld
	RUST_TARGET_PATH=$(shell pwd) xargo build --target i386 --release
	objcopy -O binary -j .text -j .rodata -j .signature ./target/i386/release/ruxv6-bootloader ./mbr

clean:
	rm ./mbr