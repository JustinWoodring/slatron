export interface BumperAsset {
    name: string
    url: string
    description: string
    thumbnail?: string
    duration_ms?: number
    category: 'abstract' | 'tech' | 'nature' | 'urban'
}

export const AVAILABLE_BUMPER_BACKS: BumperAsset[] = [
    {
        name: "Neon City Night",
        url: "https://cdn.pixabay.com/video/2025/03/13/264469_small.mp4?download",
        description: "Cyberpunk style neon city lights",
        category: "urban"
    },
    {
        name: "Matrix Rain Effect",
        url: "https://cdn.pixabay.com/video/2020/08/21/47802-451812879_small.mp4?download",
        description: "Green digital code rain loop",
        category: "tech"
    },
    {
        name: "Starfield Warp",
        url: "https://cdn.pixabay.com/video/2019/08/01/25696-352026473.mp4?download",
        description: "Moving through stars in space",
        category: "abstract"
    },
    {
        name: "Retro Synth Grid",
        url: "https://cdn.pixabay.com/video/2025/03/06/262860_small.mp4?download",
        description: "80s retro synthwave grid",
        category: "abstract"
    },
    {
        name: "Clouds Timelapse",
        url: "https://cdn.pixabay.com/video/2021/11/09/95188-644716850_small.mp4?download",
        description: "Fast moving heavy clouds",
        category: "nature"
    },
    {
        name: "Glitch Static",
        url: "https://cdn.pixabay.com/video/2020/12/01/58010-486853030_small.mp4?download",
        description: "TV Static and Glitch noise",
        category: "tech"
    }
]
