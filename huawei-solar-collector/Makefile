TARGET_ARCH=armv7-unknown-linux-gnueabihf

.PHONY: build
build:
	cargo build --release --target=${TARGET_ARCH}

.PHONY: deploy
deploy:
	rsync -avzc --progress ./target/debug/huawei-solar-collector pi:"/home/pi/test"

.PHONY: execute
execute:
	ssh -t pi /home/pi/test/huawei-solar-collector
