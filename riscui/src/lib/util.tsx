import { useContext, Context, createContext, useState, useEffect } from "react";
import * as wasm from 'riscui-wasm-lib';

export function useProvidedContext<T>(context: Context<T | null>): T {
	const contextValue = useContext(context);
	if (contextValue === null) {
		throw Error("Context has not been provided");
	}
	return contextValue;
}

export function createProvidedContext<T>(): Context<T | null> {
	return createContext(null as T | null);
}

export function AfterWasmLoaded(props: { children: any }) {
	const [wasmLoaded, setWasmLoaded] = useState(false);
	useEffect(() => {
		(async () => {
			await wasm.default();
			setWasmLoaded(true);
		})()
	});
	
	if (wasmLoaded) {
		return props.children;
	} else {
		return <p>Loading WASM</p>
	}
}