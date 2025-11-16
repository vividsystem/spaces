import { A, createAsync, query, redirect, useParams } from "@solidjs/router";
import { Space } from "..";
import { Show, Suspense } from "solid-js";
import { ArrowLeft } from "lucide-solid";
import Table, { Column } from "~/components/Table";
import { formatBytes } from "~/lib/helpers";
import FileUploadDialog from "~/components/FileUploadDialog";

export interface File {
	id: string,
	space_id: string,
	original_filename: string,
	file_size_bytes: number,
	mime_type?: string
	upload_date: Date,
	last_accessed?: Date,
	download_count: number,
	checksum: string
}
const getSpaceWithFiles = query(async () => {
	const params = useParams();

	const res = await fetch(`${import.meta.env.VITE_BACKEND_URL!}/api/spaces/${params.id}`)
	if (!res.ok) {
		throw redirect("/")
	}

	const space = await res.json() as Space;
	const res_files = await fetch(`${import.meta.env.VITE_BACKEND_URL!}/api/spaces/${space.id}/files`)
	if (!res.ok) {
		throw redirect("/")
	}
	const files = await res_files.json() as File[];
	return { space, files }

}, "getSpaceWithFiles")

export default function SpacePage() {
	const spaceFiles = createAsync(() => getSpaceWithFiles());


	const files_cols: Column<File>[] = [
		{
			header: () => "Filename",
			accessor: (item, _) => item.original_filename
		},
		{
			header: () => "Size",
			accessor: (item, _) => formatBytes(item.file_size_bytes)
		},
		{
			header: () => "Uploaded at",
			accessor: (item, _) => new Date(item.upload_date).toISOString()
		},
		{
			header: () => "",
			accessor: (item, _) => (
				<a href={`${import.meta.env.VITE_BACKEND_URL}/api/files/${item.id}/download`}>
					Download
				</a>
			)
		}

	]

	return (
		<main class="text-rose-800 p-4">
			<A href="/">
				<div class="flex flex-row items-center py-2">
					<ArrowLeft class="stroke-rose-950 lg:size-8" />
					<h1 class="lg:text-4xl text-rose-950"> Go back</h1>
				</div>
			</A>
			<Suspense fallback={"LOADING"}>
				<Show when={spaceFiles()}>
					<div class="flex flex-row justify-between py-4">
						<h1 class="lg:text-5xl">{spaceFiles()!.space.name}</h1>
						<FileUploadDialog spaceID={spaceFiles()!.space.id} />
					</div>
				</Show>
				<Show when={spaceFiles()?.space.description}>
					<p class="text-3xl text-gray-400 py-2">{spaceFiles()!.space.description}</p>
				</Show>
			</Suspense>
			<Show when={spaceFiles()}>
				<Table data={spaceFiles()!.files} columns={files_cols} />
			</Show>
		</main>
	)
}
