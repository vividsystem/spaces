import { A } from "@solidjs/router";
import { createResource, For } from "solid-js";
import SpaceCard from "~/components/SpaceCard";

export interface Space {
	id: string,
	name: string,
	description?: string,
	is_public: boolean,
	total_size_used_bytes: number,
	created_at: Date,
	updated_at: Date
	access_code?: string
}

export default function Home() {
	const [spaces] = createResource(async () => {
		const res = await fetch(`${import.meta.env.VITE_BACKEND_URL!}/api/spaces`)
		const data = await res.json()
		return data as Space[]
	})
	return (
		<main class="grid items-start p-4">
			<div class="py-4">
				<h1 class="lg:text-6xl text-white">Spaces</h1>
			</div>
			<div class="flex flex-row gap-4">
				<For each={spaces()}>
					{(space) => (
						<A href={`/spaces/${space.id}`}>
							<SpaceCard space={space} />
						</A>
					)}
				</For>
			</div>

		</main>
	);
}
