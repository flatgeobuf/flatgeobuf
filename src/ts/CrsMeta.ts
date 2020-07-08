
export default class CrsMeta {
    org: string
    code: number
    name: string
    description: string
    wkt: string
    constructor(org: string, code: number, name: string, description: string, wkt: string) {
        this.org = org
        this.code = code
        this.name = name
        this.description = description
        this.wkt = wkt
    }
}
