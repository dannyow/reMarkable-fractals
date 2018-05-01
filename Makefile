DEVICE_IP ?= "10.11.99.1"
APP ?= remarkable-fractals

build:
	cargo build --release --target=armv7-unknown-linux-gnueabihf

test:
	# Notice we aren't using the armv7 target here
	cargo test

run: build
	ssh root@$(DEVICE_IP) 'kill -9 `pidof $(APP)` || true; systemctl stop xochitl || true'
	scp ./target/armv7-unknown-linux-gnueabihf/release/$(APP) root@$(DEVICE_IP):~/
	ssh root@$(DEVICE_IP) './$(APP)'

start-xochitl:
	ssh root@$(DEVICE_IP) 'kill -9 `pidof $(APP)` || true; systemctl start xochitl'

