// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {

  WalletButton
} from '@roochnetwork/rooch-sdk-kit'
export const WalletConnect = () => {


  return (
    <>
      <WalletButton />
      {/* <Button
        variant="default"
        size="default"
        className="rounded-lg ml-2 h-auto shadow-custom bg-white hover:bg-zinc-200 dark:bg-zinc-800 dark:hover:bg-zinc-800/75 dark:shadow-muted/80"
        onClick={() => setSelectWalletDialog(true)}
      >
        <div className="flex items-center justify-center gap-x-2">
          <WalletIcon className="h-[1rem] w-[1rem] md:h-[1.2rem] md:w-[1.2rem] rotate-0 scale-100 transition-all text-teal-600 dark:text-teal-400" />
          <div className="flex items-center justify-center gap-x-1 bg-gradient-to-r bg-clip-text font-black text-transparent from-teal-600 via-purple-600 to-orange-600 dark:from-teal-400 dark:via-purple-400 dark:to-orange-400">
            {wallet ? formatAddress(wallet.getBitcoinAddress().toStr()) : 'Connect Wallet'}
          </div>
        </div>
      </Button> */}

      {/* <Dialog open={selectWalletDialog} onOpenChange={setSelectWalletDialog}>
        <DialogTrigger asChild />
        <DialogContent
          className={cn(
            'sm:max-w-[425px] overflow-hidden',
            isConnecting ? 'border-zinc-500 dark:border-zinc-600' : '',
          )}
        >
          {isConnecting && (
            <div className="absolute inset-0 flex items-center justify-center bg-zinc-500 bg-opacity-70 z-10">
              <LoadingSpinner />
            </div>
          )}
          <DialogHeader>
            <DialogTitle className="text-2xl text-center">Connect Wallet</DialogTitle>
          </DialogHeader>
          {wallets.map((wallet) => (
            <Card
              key={wallet.getName()}
              onClick={() => handleConnectSpecificWallet(wallet)}
              className="bg-secondary cursor-pointer hover:border-primary/20 transition-all"
            >
              <CardHeader className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center justify-start">
                    <img
                      src={walletsMaterialMap.get(wallet.getName())!.icon}
                      alt={walletsMaterialMap.get(wallet.getName())!.description}
                      className="h-[2rem] w-[2rem] rotate-0 scale-100 mr-4 object-cover"
                    />
                    <div>
                      <CardTitle>
                        <span className="flex items-center justify-start">
                          <p>{capitalizeFirstLetter(wallet.getName())} Wallet</p>
                          {wallet?.getName() === wallet.getName() && (
                            <Badge
                              variant="outline"
                              className="ml-2 border-teal-400 text-teal-400 hover:bg-teal-400/10 py-0 px-0.5 rounded-md"
                            >
                              Current
                            </Badge>
                          )}
                        </span>
                      </CardTitle>
                      <CardDescription>Connecting using {wallet.getName()} Wallet</CardDescription>
                    </div>
                  </div>
                  <div className="flex items-center justify-center gap-1">
                    {walletsMaterialMap.get(wallet.getName())!.types.map((type) => (
                      <img
                        key={type}
                        src={`/icon-${type}.svg`}
                        alt="Unisat Icon"
                        className="h-[1.1rem] w-[1.1rem] rotate-0 scale-100"
                      />
                    ))}
                  </div>
                </div>
              </CardHeader>
            </Card>
          ))}
        </DialogContent>
      </Dialog> */}
    </>
  )
}
