import { A } from "@solidjs/router";
import GithubIcon from "./GithubIcon";

export default function Footer() {
	return (
		<footer class="fixed bottom-0 left-0 right-0 flex flex-row items-center justify-between p-4 text-orange-800">
			<span>made by <A href="https://github.com/vividsystem">vividsystem</A></span>
			<A href="https://github.com/vividsystem/spaces"><GithubIcon class="stroke-orange-800 fill-orange-800 size-8" /></A>
		</footer>
	)
}
