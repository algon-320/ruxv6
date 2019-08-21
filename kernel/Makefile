kernel: src/*.rs  src/*.S i386.json kernel.ld
	RUST_TARGET_PATH=$(shell pwd) xargo build --target i386 --release
	cp ./target/i386/release/ruxv6-kernel ./kernel

clean:
	rm kernel