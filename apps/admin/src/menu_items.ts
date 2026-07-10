type MenuItem = {
  label: string
  items: {
    id: string
    label: string
    link?: string
    childrens?: {
      id: string
      link?: string
      label: string
    }[]
  }[]
}

export const MENU_ITEMS: MenuItem[] = [
  {
    label: '메뉴 1',
    items: [
      {
        id: 'dropdown',
        label: '드롭다운 메뉴',
        childrens: [
          {
            id: 'dropdown-menu1',
            label: '메뉴 1',
            link: '',
          },
          {
            id: 'dropdown-menu2',
            label: '메뉴 2',
            link: '',
          },
          {
            id: 'dropdown-menu3',
            label: '메뉴 3',
            link: '',
          },
        ],
      },
      {
        id: 'menu1',
        label: '메뉴 1',
        link: '',
      },
      {
        id: 'menu2',
        label: '메뉴 2',
        link: '',
      },
      {
        id: 'menu3',
        label: '메뉴 3',
        link: '',
      },
    ],
  },
  {
    label: '메뉴 2',
    items: [
      {
        id: 'menu1',
        label: '메뉴 1',
        link: '',
      },
      {
        id: 'menu2',
        label: '메뉴 2',
        link: '',
      },
      {
        id: 'menu3',
        label: '메뉴 3',
        link: '',
      },
    ],
  },
]
