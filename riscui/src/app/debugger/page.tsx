"use client";

import { useState, useContext } from 'react';
import { ExecutionStateProvider, useExecution, ExecutionState } from '../../lib/execution.tsx'
import { useImmerReducer } from 'use-immer'
import { AfterWasmLoaded, useProvidedContext } from '@/lib/util.tsx';

const SOURCE = `
main:
	li a0 16
	li a1 100
	sw a1 0(a0)
    # load the value of n into a0
    li a0 2

    # load the value of exp into a1
    li a1 10

    # call ex3
    jal ex3

    # prints the output of ex3
    mv a1 a0
    li a0 1
    ecall # Print Result

    # exits the program
    li a0 17
    li a1 0
    ecall

ex3:
    # this function is a recursive pow function
    # a0 contains the base
    # a1 contains the power to raise to
    # the return value should be the result of a0^a1
    #     where ^ is the exponent operator, not XOR
    addi sp sp -4
    sw ra 0(sp)

    # return 1 if a0 == 0
    beq a1 x0 ex3_zero_case

    # otherwise, return ex3(a0, a1-1) * a0
    mv t0 a0      # save a0 in t0
    addi a1 a1 -1 # decrement a1
    
    addi sp sp -4
    sw t0 0(sp)
    jal ex3       # call ex3(a0, a1-1)
    lw t0 0(sp)
    addi sp sp 4

    mul a0 a0 t0  # multiply ex3(a0, a1-1) by t0
                  # (which contains the value of a0)

    j ex3_end

ex3_zero_case:
    li a0 1

ex3_end:
    lw ra 0(sp)
    addi sp sp 4
    ret
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
			<ExecutionControls/>
			<InstructionList/>
		</div>
	)
}

function InstructionList() {
	const [execution, executionDispatch] = useExecution();
	
	let instructions = new Array();
	for (let i = 0; i < execution.instructions.length; i++) {
		let instruction = execution.instructions[i];
		let text = execution.instructionTexts[i];
		let hex = formatHexWord(instruction);
		instructions.push(<Instruction key={i} text={text} hex={hex} active={execution.activeIndex == i} />);
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

function parseNumber(s: string): number | null {
	const n = Number.parseInt(s);
	if (Number.isNaN(n)) {
		return null;
	} else {
		return n;
	}
}

function MemoryPane() {
	const [execution, dispatch] = useExecution();
	
	const [rawStart, setRawStart] = useState("0x00000000");
	
	const cellStride = 4;
	const nColumns = 4;
	const rowStride = cellStride * nColumns;
	const nRows = 16;
	let values = getMemoryFormatted(execution.memoryView, 4, MemoryFormat.Hex);
	
	let rows = new Array();
	for (let i = 0; i < nRows; i++) {
		let row = new Array();
		let rowStartAddr = execution.memoryViewStart + i * nColumns * cellStride;
		row.push(<td key={-1}><span>{formatHexWord(rowStartAddr)}: </span></td>)
		for (let j = 0; j < nColumns; j++) {
			row.push(<td key={j}><span className="memory-cell">{values[i * nColumns + j]}</span></td>);
		}
		rows.push(<tr key={i}>{row}</tr>)
	}
	
	function onInputChange(e: any) {
		setRawStart(e.target.value);
		let start = parseNumber(e.target.value);
		if (start !== null) {
			const actualStart = Math.min(start - (start % rowStride), 1024 * 1024);
			const len = Math.min(nRows * nColumns * cellStride, 1024 * 1024 - actualStart);
			dispatch({ action: 'updateMemoryView', start: actualStart, len: len });
		}
	}
	
	return <div className="memory-pane">
		<h2>Memory</h2>
		<input type="text" onChange={onInputChange} value={rawStart} />
		<table>
			<tbody>
				{rows}
			</tbody>
		</table>
	</div> 
}