kernel: src/*.rs  src/*.S ../i386.json kernel.ld
	RUSTFLAGS="-C link-arg=-Tkernel.ld" cargo xbuild --release
	cp ./target/i386/release/ruxv6-kernel ./kernel

clean:
	cargo clean
	rm kernel
