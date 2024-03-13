import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Wallet } from 'lucide-react'
import { useTranslation } from 'react-i18next'

export const WalletConnect = () => {
  const { t } = useTranslation()

  return (
    <>
      {/* desktop */}
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="default" size="sm" className="hidden md:flex ml-2">
              <div className="flex items-center justify-center gap-x-2">
                <Wallet className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
                {t('WalletConnect.connectWallet')}
              </div>
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{t('WalletConnect.connectWalletTip')}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>

      {/* mobile */}
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="default" size="sm" className="md:hidden flex ml-2">
              <div className="flex items-center justify-center gap-x-2">
                <Wallet className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
                {t('WalletConnect.connectWalletOnMobile')}
              </div>
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{t('WalletConnect.connectWalletTip')}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </>
  )
}
