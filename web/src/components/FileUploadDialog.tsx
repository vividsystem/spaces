import { Dialog } from "@kobalte/core/dialog"
import { action } from "@solidjs/router"
import { Plus } from "lucide-solid"
import { Setter } from "solid-js"
import { createSignal, JSX, Show } from "solid-js"
import { File } from "~/routes/spaces/[id]"
export default function FileUploadDialog(props: { spaceID: string }) {
	const [open, setOpen] = createSignal(false)
	const [message, setMessage] = createSignal("")

	const handleSubmit = async (e: Event) => {
		e.preventDefault()
		const form = e.target as HTMLFormElement
		const formData = new FormData(form)
		try {
			const res = await fetch(`${import.meta.env.VITE_BACKEND_URL}/api/spaces/${props.spaceID}/files`, {
				method: "POST",
				body: formData,
				// Don't set Content-Type header - browser will set it with correct boundary
			})

			if (!res.ok) {
				console.error(res)
				const content = await res.text()
				setMessage(content)
			} else {
				// const files = await res.json() as File[]
				setMessage("Files uploaded successfully!")
				setOpen(false)
			}
		} catch (error) {
			console.error(error)
			setMessage("Upload failed: " + String(error))
		}
	}

	return (
		<Dialog open={open()}>
			<Dialog.Trigger onClick={() => setOpen(true)}>
				<div>
					<Plus class="size-12 stroke-white" />
				</div>
			</Dialog.Trigger>
			<Dialog.Portal>
				<Dialog.Overlay class="fixed inset-0 backdrop-blur-lg z-4999" onClick={() => setOpen(false)} />

				<Dialog.Content class="flex flex-col gap-4 p-4">

					<div class="isolate flex flex-col gap-4 p-4 mx-auto my-auto border-2 border-white text-white rounded-md z-5000">
						<Show when={message()}>
							<p>{message()}</p>
						</Show>
						<form onSubmit={handleSubmit}>
							<input type="file" name="files" multiple />
							<button class="bg-white py-2 px-4 text-gray-800" type="submit">Upload</button>
						</form>

					</div>
				</Dialog.Content>
			</Dialog.Portal>
		</Dialog>
	)
}
