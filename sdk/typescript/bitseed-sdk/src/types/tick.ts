import { InscriptionID } from './generator'

export type Tick = {
  tick: string
  max: number
  generator: InscriptionID
  repeat: number
  deploy_args: Array<string>
}
