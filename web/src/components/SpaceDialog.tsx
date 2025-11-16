import { Dialog } from "@kobalte/core/dialog"
import { Plus } from "lucide-solid"
import { createSignal, Setter } from "solid-js"
import { Space } from "~/routes"
export default function SpaceDialog(props: { mutateSpace: Setter<Space[] | undefined> }) {
	const [name, setName] = createSignal("")
	const [desc, setDesc] = createSignal("")
	const [open, setOpen] = createSignal(false)


	const createNew = async () => {
		const spaceName = name();
		const spaceDescription = desc()

		if (spaceName == "") {
			return
		}

		const res = await fetch(`${import.meta.env.VITE_BACKEND_URL}/api/spaces`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({ name: spaceName, description: spaceDescription || undefined })
		})

		if (!res.ok) {
			console.error(res)
			console.error(await res.json())
		} else {
			const space = await res.json() as Space;
			props.mutateSpace((prev: Space[] | undefined) => prev ? [space, ...prev] : [space])
		}



		setName("")
		setDesc("")
		setOpen(false)
		return
	}

	return (
		<Dialog open={open()}>
			<Dialog.Trigger onClick={() => setOpen(true)}>
				<div>
					<Plus class="size-12 stroke-rose-800" />
				</div>
			</Dialog.Trigger>
			<Dialog.Portal>
				<Dialog.Overlay class="fixed inset-0 backdrop-blur-lg z-4999" onClick={() => setOpen(false)} />

				<Dialog.Content class="flex flex-col gap-4 p-4">

					<div class="isolate flex flex-col gap-4 p-4 mx-auto my-auto border-2 border-white h-1/2 w-1/2 text-white rounded-md z-5000">
						<input type="text" placeholder="Space name..." class="text-xl border-2 border-gray-700" value={name()} onInput={(ev) => setName(ev.currentTarget.value)} />
						<input type="text" placeholder="Description(optional)..." class="text-xl border-2 border-gray-700" value={desc()} onInput={(ev) => setDesc(ev.currentTarget.value)} />

						<button class="bg-white py-2 px-4 text-gray-800" onClick={createNew}>Create</button>

					</div>
				</Dialog.Content>
			</Dialog.Portal>
		</Dialog>
	)
}
