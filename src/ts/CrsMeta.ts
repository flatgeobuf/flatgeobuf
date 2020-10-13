
export default class CrsMeta {
    constructor(
        public org: string | null,
        public code: number,
        public name: string | null,
        public description: string | null,
        public wkt: string | null,
        public code_string: string | null) {
    }
}
