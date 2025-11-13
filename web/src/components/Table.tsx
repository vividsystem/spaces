import { For, JSX, Show, createMemo } from "solid-js";
export interface Column<T> {
	header: () => JSX.Element;
	accessor: (item: T, index: number) => JSX.Element;
}
interface TableProps<T> {
	data: T[];
	columns?: Column<T>[];
}


function Table<T extends Record<string, any>>(props: TableProps<T>) {
	const derivedColumns = createMemo<Column<T>[]>(() => {
		if (props.columns && props.columns.length > 0) return props.columns;
		const keys = props.data.length > 0 ? Object.keys(props.data[0]) as (keyof T)[] : [];

		return keys.map((key) => ({
			header: () => String(key),
			accessor: (item: T) => item[key],
			render: (item: T, index: number) => (String(item))
		}));
	});


	return (
		<table class="divide-y divide-gray-200 table-fixed rounded-lg">
			<thead class="bg-gray-900 sticky top-0 border-2 border-white">
				<tr>
					<For each={derivedColumns()}>{(col) => (
						<th
							scope="col"
							class="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider"
						>
							{col.header()}
						</th>
					)}</For>
				</tr>
			</thead>
			<tbody class="bg-gray-700 divide-y divide-gray-900 w-full border-2 border-transparent max-w-full">
				<For each={props.data}>{(row, i) => (
					<tr class="even:bg-gray-800 odd:bg-gray-700">
						<For each={derivedColumns()}>{(col) => (
							<td class="px-4 py-4 whitespace-nowrap text-sm text-gray-300 w-fit">
								{col.accessor(row, i())}
							</td>
						)}</For>
					</tr>
				)}</For>


				<Show when={props.data.length === 0}>
					<tr>
						<td colspan={derivedColumns().length} class="p-4 text-2xl text-center text-gray-500">No data available.</td>
					</tr>
				</Show>
			</tbody>
		</table>
	);
}

export default Table; 
