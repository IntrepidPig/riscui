import { Dispatch, createContext, useState, useEffect } from 'react';
import { useImmerReducer } from 'use-immer';
import { Draft } from 'immer';
import { createProvidedContext, useProvidedContext } from './util';
import * as wasm from 'riscui-wasm-lib';
//import { Machine, compile, test, initSync } from 'riscui-wasm-lib';

type ExecutionState = {
	machine: wasm.Machine,
	instructions: Uint32Array,
	instructionTexts: string[],
	activeIndex: number,
	registers: Int32Array,
	memoryViewStart: number,
	memoryViewLen: number,
	memoryView: ArrayBuffer,
	output: string,
}

function loadSource(source: string): ExecutionState {
	let compiled = wasm.compile(source);
	let instructions = new Uint32Array(compiled.length);
	let instructionTexts = new Array();
	for (let i = 0; i < compiled.length; i++) {
		instructions[i] = compiled[i].code;
		instructionTexts.push(compiled[i].text);
	}
	let machine = wasm.Machine.new(1024 * 1024);
	return {
		machine,
		instructions,
		instructionTexts,
		activeIndex: 0,
		registers: machine.get_registers(),
		memoryViewStart: 0,
		memoryViewLen: 1024 * 1024 / (16 * 4 * 4),
		memoryView: machine.get_memory_view(0, 1024 * 1024 / (16 * 4 * 4)).buffer,
		output: "",
	};
}

export function ExecutionStateProvider(props: { source: string, children: any }) {
	const [executionState, dispatch] = useImmerReducer(executionStateReducer, null, (arg) => {
		return loadSource(props.source);
	});
	
	return <ExecutionState.Provider value={executionState}>
		<ExecutionDispatcher.Provider value={dispatch}>
			{props.children}
		</ExecutionDispatcher.Provider>
	</ExecutionState.Provider>
}

export function executionStateReducer(draft: Draft<ExecutionState>, action: any) {
	console.log("dispatch action " + JSON.stringify(action));
	
	function reload() {
		draft.activeIndex = draft.machine.get_instruction_index();
		draft.registers = draft.machine.get_registers();
		draft.memoryView = draft.machine.get_memory_view(draft.memoryViewStart, draft.memoryViewLen).buffer;
	}
	
	switch (action.action) {
		case 'reload': {
			reload();
			break;
		}
		case 'step': {
			if (draft.activeIndex >= draft.instructions.length) {
				break;
			}
			let result = draft.machine.exec(draft.instructions[draft.activeIndex]);
			reload();
			if (result !== undefined) {
				// ecall same as venus: https://github.com/kvakil/venus/wiki/Environmental-Calls
				let a1 = result.a1;
				switch (result.a0) {
					case 1: {
						draft.output += a1.toString();
						break;
					}
					case 4: {
						let full = new Uint8Array(64 * 1024);
						let end = false;
						for (let i = 0; i < 64 && !end; i++) {
							let buf = draft.machine.get_memory_view(a1 + 1024 * i, 1024);
							for (let j = 0; j < 1024; j++) {
								let byte = buf.at(1024 * i + j);
								if (byte == 0) {
									full.set(buf.slice(0, j), 1024 * i);
									end = true;
								}
							}
							
							if (end) {
								break;
							} else {
								full.set(buf, 1024 * i);
							}
						}
						if (!end) {
							console.error("didn't find end of string within 64KiB");
						}
						for (let i = 0; i < full.length && full.at(i) !== 0; i++) {
							let val = full.at(i)!;
							if (32 <= val && val < 127) {
								draft.output += String.fromCharCode(val);
							} else {
								draft.output += "\\x" + val.toString(16).padStart(2, '0');
							}
						}
						break;
					}
					case 9: {
						throw Error("unimplemented");
						break;
					}
					case 10: {
						// exit
						throw Error("unimplemented");
					}
					case 11: {
						if (32 <= a1 && a1 < 127) {
							draft.output += String.fromCharCode(a1);
						} else {
							draft.output += "\\x" + a1.toString(16).padStart(2, '0');
						}
					}
					case 17: {
						// exit2, code in a1
						throw Error("unimplemented");
					}
				}
			}
			break;
		}
		case 'reset': {
			draft.machine = wasm.Machine.new(1024 * 1024);
			draft.output = "";
			reload();
			break;
		}
		case 'updateMemoryView': {
			draft.memoryViewStart = action.start;
			draft.memoryViewLen = action.len;
			draft.memoryView = draft.machine.get_memory_view(action.start, action.len).buffer;
			break;
		}
		default: {
			throw Error("Unknown action: " + action.action);
		}
	}
	console.log("new execution state: " + JSON.stringify(draft));
	return draft;
}

export const ExecutionState = createProvidedContext<ExecutionState>();
export const ExecutionDispatcher = createProvidedContext<Dispatch<any>>();

export function useExecution(): [ExecutionState, Dispatch<any>] {
	const executionState = useProvidedContext(ExecutionState);
	const executionDispatch = useProvidedContext(ExecutionDispatcher);
	return [executionState, executionDispatch];
}