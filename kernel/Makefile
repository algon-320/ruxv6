kernel: src/*.rs  src/*.S ../i386.json kernel.ld
	RUSTFLAGS="-C link-arg=-Tkernel.ld" cargo xbuild --release
	cp ./target/i386/release/ruxv6-kernel ./kernel

kernel-debug: src/*.rs  src/*.S ../i386.json kernel.ld
	RUSTFLAGS="-C link-arg=-Tkernel.ld" cargo xbuild
	cp ./target/i386/debug/ruxv6-kernel ./kernel-debug

clean:
	cargo clean
	rm kernel
	rm kernel-debug
