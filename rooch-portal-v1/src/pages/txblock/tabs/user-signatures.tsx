import { Copy } from 'lucide-react'

export const UserSignatures = () => {
  return (
    <div className="flex flex-col items-start justify-start gap-3">
      <div className="flex flex-col items-start justify-start gap-5 font-medium">
        {/* Schema */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Schema:</span>
          </div>
          <span className="text-gray-800 dark:text-gray-50 tracking-tight">
            <span>ED25519</span>
          </span>
        </div>

        {/* Amount */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Schema:</span>
          </div>
          <div className="border border-accent dark:border-muted-foreground/15 dark:bg-blue-950 py-0.5 px-2 rounded-lg text-blue-500 dark:text-blue-300 tracking-tight hover:underline cursor-pointer">
            <span className="flex items-center justify-start gap-1 tracking-tight font-mono">
              <p>0x9b1886b1c9e6107afbb10a4d2a01dbe318776b82021b879007631496919365cb</p>
              <Copy className="w-3 h-3 text-muted-foreground" />
            </span>
          </div>
        </div>

        {/* Schema */}
        <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
          <div className="w-36">
            <span>Signature:</span>
          </div>
          <span className="text-gray-800 dark:text-gray-50 tracking-tight">
            <span>
              U8o3TDONsdB6GT9I6NBhgVp5pOXK8n0xUGPkP4tTyeVXExS1J9mNU1r76lep/01hWfj3qZNfYV/o01P/atbfCQ==
            </span>
          </span>
        </div>
      </div>
    </div>
  )
}
