import { Show } from "solid-js";
import { formatBytes } from "~/lib/helpers";
import { Space } from "~/routes";

export default function SpaceCard(props: { space: Space }) {

	return (
		<div class="bg-rose-950 rounded-xl p-4 flex flex-col items-start h-fill">

			<h1 class="text-gray-300 lg:text-5xl">{props.space.name}</h1>
			<Show when={props.space.description}>
				<p class="text-gray-600 lg:text-3xl">{props.space.description}</p>
			</Show>
			<p class="text-gray-600 lg:text-3xl">Size: {formatBytes(props.space.total_size_used_bytes)}</p>
		</div>
	)
}
