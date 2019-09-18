mbr: src/*.rs src/*.S ../i386.json ./bootloader.ld
	RUSTFLAGS="-C link-arg=-Tbootloader.ld" cargo xbuild --release
	objcopy -O binary -j .text -j .rodata -j .signature ./target/i386/release/ruxv6-bootloader ./mbr

clean:
	cargo clean
	rm ./mbr
