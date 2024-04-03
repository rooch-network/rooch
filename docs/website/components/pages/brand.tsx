import React, { useMemo, useState } from 'react'
import { ROOCH_ASSETS, ROOCH_ASSETS_ZIP_NAME } from '../../data/assets'
import axios from 'axios'
import { useRouter } from 'next/router'
import JSZip from 'jszip'
import FileSaver from 'file-saver'

const Brand = () => {
  const basePath = useRouter().basePath
  const cachedFiles = useMemo(() => new Map(), [])
  const [isDownload, setIsDownload] = useState(false)
  const getFileNameForPath = (path) => path.slice(path.lastIndexOf('/') + 1)

  const en = useRouter().locale === 'en-US'

  const downloadAll = () => {
    setIsDownload(true)
    const zip = new JSZip()

    let promises = Object.entries(ROOCH_ASSETS)
      .map(([_, assets]) => {
        const svgAssets = 'svg' in assets ? assets.svg : []
        const allAssets = [...assets.png, ...svgAssets]
        return allAssets.map((path) => {
          let url = basePath + path
          return getFile(url).then((data: any) => {
            zip.file(getFileNameForPath(path), data, { binary: true })
          })
        })
      })
      .flat()
    Promise.all(promises).then(() => {
      zip.generateAsync({ type: 'blob' }).then((content) => {
        FileSaver.saveAs(content, ROOCH_ASSETS_ZIP_NAME)
        setIsDownload(false)
      })
    })
  }

  const download = (url) => {
    const downloadLink = document.createElement('a')
    downloadLink.href = url
    downloadLink.download = getFileNameForPath(url)
    downloadLink.click()
  }

  const getFile = (url) => {
    return new Promise((resolve, reject) => {
      axios({
        method: 'get',
        url,
        responseType: 'blob',
      })
        .then((response) => {
          const file = cachedFiles.get(url) || response.data
          cachedFiles.set(url, file)
          resolve(file)
        })
        .catch((error) => {
          reject(error.toString())
        })
    })
  }

  return (
    <>
      <p className="text-2xl mt-8">{en ? 'Assets' : '资源'}</p>
      <button
        className="w-80 rounded-3xl mt-6 p-2 border hover:border-black hover:bg-black hover:text-white"
        onClick={downloadAll}
      >
        {isDownload
          ? en
            ? 'Downloading...'
            : '下载中...'
          : en
          ? 'DOWNLOAD ASSETS (.ZIP FILE)'
          : '下载资源 (.ZIP 文件)'}
      </button>

      {Object.entries(ROOCH_ASSETS).map(([category, assets]) => (
        <div key={category} className="border-b pt-10">
          <h3 className="text-2xl">{category.toUpperCase()}</h3>
          {assets.png.map((pngUrl, index) => {
            const svgUrls = 'svg' in assets ? assets.svg : [] // 类型断言
            const svgUrl = svgUrls[index]

            return (
              <div key={pngUrl} className="flex flex-row border-t mt-2">
                <div className="basis-1/2">
                  <div className="flex flex-row mt-6">
                    <button
                      className="basis-1/3 rounded-3xl p-2 border hover:border-black hover:bg-black hover:text-white"
                      onClick={() => download(basePath + pngUrl)}
                    >
                      PNG
                    </button>
                    {svgUrl && (
                      <button
                        className="basis-1/3 ml-8 rounded-3xl p-2 border hover:border-black hover:bg-black hover:text-white"
                        onClick={() => download(basePath + svgUrl)}
                      >
                        SVG
                      </button>
                    )}
                  </div>
                </div>
                <div className="basis-1/2 mt-6 mb-6 bg-white p-3">
                  <img className="mx-auto" src={basePath + pngUrl} alt={`${category} image`} />
                </div>
              </div>
            )
          })}
        </div>
      ))}
    </>
  )
}

export default Brand
