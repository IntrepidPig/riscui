"use client";

import { useState, useContext } from 'react';
import { ExecutionStateProvider, useExecution, ExecutionState } from '../../lib/execution.tsx'
import { useImmerReducer } from 'use-immer'
import { AfterWasmLoaded, useProvidedContext } from '@/lib/util.tsx';

const SOURCE = `
addi t0 x0 0
addi t1 x0 1
addi t5 x0 7
start:
add t2 t0 t1
addi t0 t1 0
addi t1 t2 0
addi t5 t5 -1
sw t0 0(x0)
bge t5 x0 start
addi t3 x0 -1
sw t1 4(x0)
`;

export default function Debugger() {
	let source = SOURCE;
	
	return (
		<AfterWasmLoaded>
			<ExecutionStateProvider source={source}>
				<div className="debugger">
					<Execution/>
					<div className="registers-and-output-panes">
						<RegistersPane/>
						<OutputPane/>
					</div>
					<MemoryPane/>
				</div>
			</ExecutionStateProvider>
		</AfterWasmLoaded>
	);
}

function RegistersPane() {
	const [execution, dispatch] = useExecution();
	let rows = new Array();
	for (let i = 0; i < 16; i++) {
		let ip1 = i;
		let ip2 = 16 + i;
		rows.push(<tr key={i}>
			<td><Register index={ip1} value={execution.registers[ip1]}/></td>
			<td><Register index={ip2} value={execution.registers[ip2]}/></td>
		</tr>)
	}
	return <div className="registers-pane">
		<h2>Registers</h2>
		<table>
			<tbody>
				{rows}
			</tbody>
		</table>
	</div>
}

function Register(props: { index: number, value: number }) {
	return <div className="register">
		<div className="name">x{props.index} =</div>
		<div className="value"><input type="text" disabled={true} value={props.value.toString()}/></div>
	</div>
}

function OutputPane() {
	return <div className="output-pane">
		<h2>Output</h2>
		<textarea disabled={true}>
			
		</textarea>
	</div>
}

function Execution() {
	return (
		<div className="execution-pane">
			<h2>Execution</h2>
			<InstructionList/>
			<ExecutionControls/>
		</div>
	)
}

function InstructionList() {
	const [execution, executionDispatch] = useExecution();
	
	let instructions = new Array();
	for (let i = 0; i < execution.instructions.length; i++) {
		let instruction = execution.instructions[i];
		let hex = formatHexWord(instruction);
		instructions.push(<Instruction key={i} text={instruction.toString()} hex={hex} active={execution.activeIndex == i} />);
	}
	
	return (
		<table className="instruction-list">
			<thead>
				<tr>
					<td style={{width:'10%'}}>Break</td>
					<td>Instruction</td>
					<td>Hex</td>
				</tr>
			</thead>
			<tbody>
				{instructions}
			</tbody>
		</table>
	)
}

type InstructionProps = {
	text: string,
	hex?: string,
	active?: boolean,
}

function Instruction(props: InstructionProps) {
	const [broken, setBroken] = useState(false);
	function handleClick() {
		setBroken(!broken);
	}
	const className = props.active ? "instruction on" : "instruction"
	return <tr className={className}>
		<td><BreakpointMark broken={broken} onClick={handleClick} /></td>
		<td className="inst-text">
			{props.text}
		</td>
		<td className="inst-hex">
			{props.hex}
		</td>
	</tr>;
}

function BreakpointMark(props: { broken: boolean, onClick: () => void }) {
	const className = props.broken ? "breakpoint-mark on" : "breakpoint-mark";
	return <button className={className} onClick={props.onClick}></button>
}

async function sleep(ms: number) {
	return new Promise<void>((resolve) => {
		setTimeout(() => {
			resolve();
		}, ms);
	});
}

function ExecutionControls() {
	const [execution, executionDispatch] = useExecution();
	const [stopper, setStopper] = useState<any>(null);
	
	function handleRun() {
		// can't believe this works tbh
		let running = true;
		setStopper(() => (() => {
			running = false;
		}));
		(async () => {
			let activeIndex = execution.activeIndex;
			while (activeIndex < execution.instructions.length && running) {
				executionDispatch({ action: 'step' });
				activeIndex = execution.machine.get_instruction_index();
				await sleep(10);
			}
			setStopper(null);
			executionDispatch({ action: 'reload' });
		})();
	}
	
	function handlePause() {
		stopper();
		setStopper(null);
		executionDispatch({ action: 'reload' });
	}
	
	let play;
	if (stopper === null) {
		play = <ExecutionButton name="Run" onClick={handleRun}/>;
	} else {
		play = <ExecutionButton name="Pause" onClick={handlePause}/>;
	}
	
	return <div className="execution-controls">
		{play}
		<ExecutionButton name="Step" onClick={() => { executionDispatch({ action: 'step' })}}/>
		<ExecutionButton name="Reset" onClick={() => { executionDispatch({ action: 'reset' })}}/>
	</div>;
}

function ExecutionButton(props: { name: string, onClick?: any }) {
	return <button className="rounded-md m-2 p-2 bg-sky-300 hover:bg-sky-500" onClick={props.onClick}>{props.name}</button>
}

function formatHexWord(addr: number): string {
	return '0x' + addr.toString(16).padStart(8, '0');
}

enum MemoryFormat {
	Hex,
	Signed,
	Unsigned,
	Ascii,
}

function getMemoryFormatted(view: ArrayBuffer, cellStride: number, format: MemoryFormat): string[] {
	let formatFunction;
	switch (format) {
		case MemoryFormat.Hex: {
			formatFunction = (val: number) => '0x' + val.toString(16).padStart(cellStride * 2, '0');
			break;
		}
		case MemoryFormat.Signed: {
			formatFunction = (val: number) => val.toString();
			break;
		}
		case MemoryFormat.Unsigned: {
			formatFunction = (val: number) => val.toString();
			break;
		}
		case MemoryFormat.Ascii: {
			formatFunction = (val: number) => {
				if (32 <= val && val < 127) {
					return String.fromCharCode(val);
				} else {
					return '\\x' + val.toString(16).padStart(2, '0');
				}
			};
			break;
		}
	}
	let arr;
	switch (cellStride) {
		case 1: {
			if (format == MemoryFormat.Signed) {
				arr = new Int8Array(view);
			} else {
				arr = new Uint8Array(view);
			}
			break;
		}
		case 2: {
			if (format == MemoryFormat.Signed) {
				arr = new Int16Array(view);
			} else {
				arr = new Uint16Array(view);
			}
			break;
		}
		case 4: {
			if (format == MemoryFormat.Signed) {
				arr = new Int32Array(view);
			} else {
				arr = new Uint32Array(view);
			}
			break;
		}
		default: {
			throw Error("unexpected cellStride " + cellStride);
		}
	}
	return Array.from(arr).map(formatFunction);
}

function MemoryPane() {
	const [execution, dispatch] = useExecution();
	
	const cellStride = 4;
	const nColumns = 4;
	const rowStride = cellStride * nColumns;
	const nRows = 16;
	const actualStart = Math.floor(execution.memory_view_start / rowStride);
	let memViewBuf = execution.machine.get_memory_view(actualStart, nRows * nColumns * cellStride).buffer;
	let values = getMemoryFormatted(memViewBuf, 4, MemoryFormat.Ascii);
	
	let rows = new Array();
	for (let i = 0; i < nRows; i++) {
		let row = new Array();
		let rowStartAddr = actualStart + i * nColumns * cellStride;
		row.push(<td key={-1}><span>{formatHexWord(rowStartAddr)}: </span></td>)
		for (let j = 0; j < nColumns; j++) {
			row.push(<td key={j}><span className="memory-cell">{values[i * nColumns + j]}</span></td>);
		}
		rows.push(<tr key={i}>{row}</tr>)
	}
	
	return <div className="memory-pane">
		<h2>Memory</h2>
		<table>
			<tbody>
				{rows}
			</tbody>
		</table>
	</div> 
}