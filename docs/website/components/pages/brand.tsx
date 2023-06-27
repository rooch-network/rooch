import React, { useMemo, useState } from "react";
import { ROOCH_ASSETS, ROOCH_ASSETS_ZIP_NANME } from "../../data/assets";
import axios from "axios";
import { useRouter } from "next/router";
import JSZip from "jszip";
import FileSaver from "file-saver";

const Brand = () => {
  let basePath = useRouter().basePath;
  let cachedFiles = useMemo(() => new Map());
  let [isDownload, setIsDownload] = useState(false);
  const getFileNameForPath = (path: string) =>
    path.slice(path.lastIndexOf("/") + 1);

  let en = useRouter().locale === "en-US";

  const downloadAll = () => {
    setIsDownload(true);
    let zip = new JSZip();

    let promises = Object.entries(ROOCH_ASSETS)
      .map((k) =>
        k[1].map((path) => {
          let url = (basePath = path);
          return getFile(url).then((data: any) => {
            zip.file(getFileNameForPath(path), data, { binary: true });
          });
        })
      )
      .flat();

    Promise.all(promises).then(() => {
      zip.generateAsync({ type: "blob" }).then((content) => {
        console.log("save");
        FileSaver.saveAs(content, ROOCH_ASSETS_ZIP_NANME);
        setIsDownload(false);
      });
    });
  };

  const download = (url: string) => {
    const downloadLink = document.createElement("a");
    downloadLink.href = url;
    downloadLink.download = url.slice(url.lastIndexOf("/") + 1);
    downloadLink.click();
  };

  const getFile = (url: string) => {
    return new Promise((resolve, reject) => {
      axios({
        method: "get",
        url,
        responseType: "blob",
      })
        .then((data) => {
          let file = cachedFiles.get(url);

          if (file) {
            console.log("cached " + url);
            resolve(file);
            return;
          }

          console.log(url);

          cachedFiles.set(url, data);
          resolve(data.data);
        })
        .catch((error) => {
          reject(error.toString());
        });
    });
  };

  return (
    <>
      <p className="text-2xl mt-8">{en ? "Assets" : "资源"}</p>
      <button
        className="w-80 rounded-3xl mt-6 p-2 border hover:border-black hover:bg-black hover:text-white"
        onClick={() => downloadAll()}
      >
        {isDownload
          ? en
            ? "Downling"
            : "下载中"
          : en
          ? "DOWNLOAD ASSETS (.ZIP FILE)"
          : "下载资源 (.ZIP 文件)"}
      </button>

      {Object.entries(ROOCH_ASSETS).map((v) => (
        <div key={v[0]} className=" border-b pt-10">
          <h3 className="text-2xl ">
            {getFileNameForPath(v[0]).toLocaleUpperCase()}
          </h3>

          {v[1].map((url) => (
            <div key={url} className="flex flex-row border-t mt-2">
              <div className="basis-1/2">
                <div className="flex flex-row mt-6">
                  <button
                    className="basis-1/3 rounded-3xl p-2 border hover:border-black hover:bg-black hover:text-white"
                    onClick={() => download(url)}
                  >
                    PNG
                  </button>
                  <button
                    disabled={true}
                    className="basis-1/3 ml-8 rounded-3xl p-2 border hover:border-black hover:bg-black hover:text-white"
                    onClick={() => download(url)}
                  >
                    SVG
                  </button>
                </div>
              </div>
              <div className=" basis-1/2 mt-6 mb-6 bg-white p-3">
                <img className="mx-auto" src={url} alt="Image" />
              </div>
            </div>
          ))}
        </div>
      ))}
    </>
  );
};

export default Brand;
