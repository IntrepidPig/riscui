"use client";

import { useState } from 'react';
import * as wasm from 'riscui-wasm-lib'


export default function TestPage() {
	console.log("TestPage");
	const [state, setState] = useState(false);
	
	return <div>
		<button onClick={() => {setState(!state)}}>Switch</button>
		<Selector tag={state}><CompA/></Selector>
	</div>
}

function Selector(props: { tag: boolean, children: any }) {
	console.log("Selector");
	if (props.tag) {
		return <div>{props.children}</div>;
	} else {
		return <p>None</p>;
	}
}

function CompA() {
	console.log("CompA");
	return <div><p>CompA</p></div>
}

function CompB() {
	console.log("CompB");
	return <div><p>CompB</p></div>
}

function WasmComponent() {
	
}