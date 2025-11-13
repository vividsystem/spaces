export function formatBytes(size: number) {
	if (size >= (10 ** 12)) {
		return `${(size / (10 ** 12)).toFixed(2)} TB`
	} else if (size >= (10 ** 9)) {
		return `${(size / (10 ** 9)).toFixed(2)} GB`
	} else if (size >= (10 ** 6)) {
		return `${(size / (10 ** 6)).toFixed(2)} MB`
	} else if (size >= 1000) {
		return `${(size / 1000).toFixed(2)} kB`
	} else {
		return `${size} B`
	}


}
