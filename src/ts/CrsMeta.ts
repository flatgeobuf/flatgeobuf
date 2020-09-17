
export default class CrsMeta {
    org: string | null
    code: number
    name: string | null
    description: string | null
    wkt: string | null
    constructor(org: string | null, code: number, name: string | null, description: string | null, wkt: string | null) {
        this.org = org
        this.code = code
        this.name = name
        this.description = description
        this.wkt = wkt
    }
}
