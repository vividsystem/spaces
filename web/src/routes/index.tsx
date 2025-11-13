import { Dialog } from "@kobalte/core/dialog";
import { A } from "@solidjs/router";
import { Plus } from "lucide-solid";
import { createResource, For } from "solid-js";
import SpaceCard from "~/components/SpaceCard";
import SpaceDialog from "~/components/SpaceDialog";

export interface Space {
	id: string,
	name: string,
	description?: string,
	is_public: boolean,
	total_size_used_bytes: number,
	created_at: string,
	updated_at: string
	access_code?: string
}

export default function Home() {
	const [spaces, { mutate }] = createResource(async () => {
		const res = await fetch(`${import.meta.env.VITE_BACKEND_URL!}/api/spaces`)
		const data = await res.json()
		return data as Space[]
	})
	return (
		<main class="grid items-start p-4">
			<div class="py-4 items-center justify-between flex flex-row">
				<h1 class="lg:text-6xl text-white">Spaces</h1>
				<SpaceDialog mutateSpace={mutate} />

			</div>
			<div class="grid lg:grid-cols-5 grid-cols-1 gap-4">
				<For each={spaces()}>
					{(space) => (
						<A href={`/spaces/${space.id}`}>
							<SpaceCard space={space} />
						</A>
					)}
				</For>
			</div>

		</main >
	);
}
