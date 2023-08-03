default:
	just list

run-dev-ui:
	cd riscui && npm run dev

build-wasm:
	cd riscui-wasm-lib && wasm-pack build -t web