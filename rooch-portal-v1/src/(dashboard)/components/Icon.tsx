import React, { useState, useEffect } from 'react'

type IconProps = {
  url: string
}

const Icon: React.FC<IconProps> = ({ url }) => {
  const [svgContent, setSvgContent] = useState('')

  useEffect(() => {
    fetch(url)
      .then((response) => response.text())
      .then((data) => {
        setSvgContent(data)
      })
  }, [url])

  return (
    <div
      className="w-8 h-8 text-gray-200 animate-spin dark:text-gray-600 fill-blue-900"
      dangerouslySetInnerHTML={{ __html: svgContent }}
    />
  )
}

export default Icon
