export enum category {
  banner = 'banner',
  wordmark = 'wordmark',
  combine = 'combine',
  logo = 'logo',
}

export enum type {
  svg = 'svg',
  png = 'png',
}

const BASE_PATH = '/logo/'

const getAssetPath = (category: category, assets: string[], type: type) => {
  const paths = assets.map((asset) => `${BASE_PATH}${category}/${asset}.${type}`)
  return paths
}

const assetsList = {
  [category.banner]: ['rooch-banner'],
  [category.wordmark]: ['rooch_black_text', 'rooch_white_text'],
  [category.combine]: [
    'rooch_black_combine_wrap',
    'rooch_color_combine_wrap',
    'rooch_color_light_combine_wrap',
    'rooch_white_combine_wrap',
    'rooch_black_combine',
    'rooch_white_combine',
    'rooch_color_combine',
  ],
  [category.logo]: [
    'rooch_circle_black_logo',
    'rooch_circle_logo',
    'rooch_color_light_logo',
    'rooch_color_logo',
    'rooch_black_logo',
    'rooch_white_logo',
  ],
}

export const ROOCH_ASSETS = {
  banner: {
    png: getAssetPath(category.banner, assetsList[category.banner], type.png),
  },
  wordmark: {
    png: getAssetPath(category.wordmark, assetsList[category.wordmark], type.png),
    svg: getAssetPath(category.wordmark, assetsList[category.wordmark], type.svg),
  },
  combine: {
    png: getAssetPath(category.combine, assetsList[category.combine], type.png),
    svg: getAssetPath(category.combine, assetsList[category.combine], type.svg),
  },
  logo: {
    png: getAssetPath(category.logo, assetsList[category.logo], type.png),
    svg: getAssetPath(category.logo, assetsList[category.logo], type.svg),
  },
}

export const ROOCH_ASSETS_ZIP_NAME = 'rooch.zip'
