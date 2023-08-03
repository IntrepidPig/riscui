import { Dispatch, createContext, useState, useEffect } from 'react';
import { useImmerReducer } from 'use-immer';
import { Draft } from 'immer';
import { createProvidedContext, useProvidedContext } from './util';
import * as wasm from 'riscui-wasm-lib';
//import { Machine, compile, test, initSync } from 'riscui-wasm-lib';

type ExecutionState = {
	machine: wasm.Machine,
	instructions: Uint32Array,
	activeIndex: number,
	registers: Int32Array,
	memory_view_start: number,
	memory_view: ArrayBuffer,
}

function loadSource(source: string): ExecutionState {
	let instructions = wasm.compile(source);
	let machine = wasm.Machine.new(1024);
	return {
		machine,
		instructions,
		activeIndex: 0,
		registers: new Int32Array(32),
		memory_view_start: 0,
		memory_view: machine.get_memory_view(0, 128).buffer,
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
	function reload() {
		console.log('reloading');
		draft.activeIndex = draft.machine.get_instruction_index();
		draft.registers = draft.machine.get_registers();
		draft.memory_view = draft.machine.get_memory_view(draft.memory_view_start, 128).buffer;
	}
	
	switch (action.action) {
		case 'reload': {
			reload();
			break;
		}
		case 'step': {
			if (draft.activeIndex >= draft.instructions.length) {
				return draft;
			}
			draft.machine.exec(draft.instructions[draft.activeIndex]);
			reload();
			break;
		}
		case 'reset': {
			draft.machine = wasm.Machine.new(1024);
			reload();
			break;
		}
		default: {
			throw Error("Unknown action: " + action.action);
		}
	}
	return draft;
}

export const ExecutionState = createProvidedContext<ExecutionState>();
export const ExecutionDispatcher = createProvidedContext<Dispatch<any>>();

export function useExecution(): [ExecutionState, Dispatch<any>] {
	const executionState = useProvidedContext(ExecutionState);
	const executionDispatch = useProvidedContext(ExecutionDispatcher);
	return [executionState, executionDispatch];
}