import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "IronCalc Documentation",
  description: "The documentation of IronCalc",
  head: [["link", { rel: "icon", href: "/favicon-32x32.png" }]],

  markdown: {
    container: {
      tipLabel: " ",
      warningLabel: " ",
      dangerLabel: " ",
      infoLabel: " ",
      detailsLabel: "Details",
    },
    math: true,
  },

  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config

    logo: {
      light: "/ironcalc-docs-logo.svg",
      dark: "/ironcalc-docs-logo-dark.svg",
      alt: "IronCalc Logo",
    },
    siteTitle: false,

    search: {
      provider: "local",
    },

    nav: [
      { text: "Blog", link: "https://blog.ironcalc.com/" },
      { text: "App", link: "https://app.ironcalc.com/" },
    ],

    sidebar: [
      { text: "About IronCalc", link: "/index.md" },
      {
        text: "Web Application",
        collapsed: true,
        items: [
          { text: "About the web application", link: "/web-application/about" },
          { text: "Importing Files", link: "/web-application/importing-files" },
          { text: "Sharing Files", link: "/web-application/sharing-files" },
          {
            text: "Name Manager",
            link: "/web-application/name-manager",
          }
        ],
      },
      {
        text: "Features",
        collapsed: true,
        items: [
          { text: "Formatting Values", link: "/features/formatting-values" },
          { text: "Using Styles", link: "/features/using-styles" },
          { text: "Keyboard Shortcuts", link: "/features/keyboard-shortcuts" },
          {
            text: "Error Types",
            link: "/features/error-types",
          },
          {
            text: "Value Types",
            link: "/features/value-types",
          },
          {
            text: "Optional Arguments",
            link: "/features/optional-arguments",
          },
          {
            text: "Units",
            link: "/features/units",
          },		  
          {
            text: "Serial Numbers",
            link: "/features/serial-numbers",
          },		  
          {
            text: "Unsupported Features",
            link: "/features/unsupported-features",
          },
        ],
      },
      {
        text: "Functions",
        collapsed: true,
        items: [
          {
            text: "Lookup and reference",
            collapsed: true,
            link: "/functions/lookup-and-reference",
            items: [
              {
                text: "ADDRESS",
                link: "/functions/lookup_and_reference/address",
              },
              {
                text: "AREAS",
                link: "/functions/lookup_and_reference/areas",
              },
              {
                text: "CHOOSE",
                link: "/functions/lookup_and_reference/choose",
              },
              {
                text: "CHOOSECOLS",
                link: "/functions/lookup_and_reference/choosecols",
              },
              {
                text: "CHOOSEROWS",
                link: "/functions/lookup_and_reference/chooserows",
              },
              {
                text: "COLUMN",
                link: "/functions/lookup_and_reference/column",
              },
              {
                text: "COLUMNS",
                link: "/functions/lookup_and_reference/columns",
              },
              {
                text: "DROP",
                link: "/functions/lookup_and_reference/drop",
              },
              {
                text: "EXPAND",
                link: "/functions/lookup_and_reference/expand",
              },
              {
                text: "FILTER",
                link: "/functions/lookup_and_reference/filter",
              },
              {
                text: "FORMULATEXT",
                link: "/functions/lookup_and_reference/formulatext",
              },
              {
                text: "GETPIVOTDATA",
                link: "/functions/lookup_and_reference/getpivotdata",
              },
              {
                text: "HLOOKUP",
                link: "/functions/lookup_and_reference/hlookup",
              },
              {
                text: "HSTACK",
                link: "/functions/lookup_and_reference/hstack",
              },
              {
                text: "HYPERLINK",
                link: "/functions/lookup_and_reference/hyperlink",
              },
              {
                text: "IMAGE",
                link: "/functions/lookup_and_reference/image",
              },
              {
                text: "INDEX",
                link: "/functions/lookup_and_reference/index",
              },
              {
                text: "INDIRECT",
                link: "/functions/lookup_and_reference/indirect",
              },
              {
                text: "LOOKUP",
                link: "/functions/lookup_and_reference/lookup",
              },
              {
                text: "MATCH",
                link: "/functions/lookup_and_reference/match",
              },
              {
                text: "OFFSET",
                link: "/functions/lookup_and_reference/offset",
              },
              {
                text: "ROW",
                link: "/functions/lookup_and_reference/row",
              },
              {
                text: "ROWS",
                link: "/functions/lookup_and_reference/rows",
              },
              {
                text: "RTD",
                link: "/functions/lookup_and_reference/rtd",
              },
              {
                text: "SORT",
                link: "/functions/lookup_and_reference/sort",
              },
              {
                text: "SORTBY",
                link: "/functions/lookup_and_reference/sortby",
              },
              {
                text: "TAKE",
                link: "/functions/lookup_and_reference/take",
              },
              {
                text: "TOCOL",
                link: "/functions/lookup_and_reference/tocol",
              },
              {
                text: "TOROW",
                link: "/functions/lookup_and_reference/torow",
              },
              {
                text: "TRANSPOSE",
                link: "/functions/lookup_and_reference/transpose",
              },
              {
                text: "UNIQUE",
                link: "/functions/lookup_and_reference/unique",
              },
              {
                text: "VLOOKUP",
                link: "/functions/lookup_and_reference/vlookup",
              },
              {
                text: "VSTACK",
                link: "/functions/lookup_and_reference/vstack",
              },
              {
                text: "WRAPCOLS",
                link: "/functions/lookup_and_reference/wrapcols",
              },
              {
                text: "WRAPROWS",
                link: "/functions/lookup_and_reference/wraprows",
              },
              {
                text: "XLOOKUP",
                link: "/functions/lookup_and_reference/xlookup",
              },
              {
                text: "XMATCH",
                link: "/functions/lookup_and_reference/xmatch",
              },
            ],
          },
          {
            text: "Cube",
            collapsed: true,
            link: "/functions/cube",
            items: [
              {
                text: "CUBEKPIMEMBER",
                link: "/functions/cube/cubekpimember",
              },
              {
                text: "CUBEMEMBER",
                link: "/functions/cube/cubemember",
              },
              {
                text: "CUBEMEMBERPROPERTY",
                link: "/functions/cube/cubememberproperty",
              },
              {
                text: "CUBERANKEDMEMBER",
                link: "/functions/cube/cuberankedmember",
              },
              {
                text: "CUBESET",
                link: "/functions/cube/cubeset",
              },
              {
                text: "CUBESETCOUNT",
                link: "/functions/cube/cubesetcount",
              },
              {
                text: "CUBEVALUE",
                link: "/functions/cube/cubevalue",
              },
            ],
          },
          {
            text: "Financial",
            collapsed: true,
            link: "/functions/financial",
            items: [
              {
                text: "ACCRINT",
                link: "/functions/financial/accrint",
              },
              {
                text: "ACCRINTM",
                link: "/functions/financial/accrintm",
              },
              {
                text: "AMORDEGRC",
                link: "/functions/financial/amordegrc",
              },
              {
                text: "AMORLINC",
                link: "/functions/financial/amorlinc",
              },
              {
                text: "COUPDAYBS",
                link: "/functions/financial/coupdaybs",
              },
              {
                text: "COUPDAYS",
                link: "/functions/financial/coupdays",
              },
              {
                text: "COUPDAYSNC",
                link: "/functions/financial/coupdaysnc",
              },
              {
                text: "COUPNCD",
                link: "/functions/financial/coupncd",
              },
              {
                text: "COUPNUM",
                link: "/functions/financial/coupnum",
              },
              {
                text: "COUPPCD",
                link: "/functions/financial/couppcd",
              },
              {
                text: "CUMIPMT",
                link: "/functions/financial/cumipmt",
              },
              {
                text: "CUMPRINC",
                link: "/functions/financial/cumprinc",
              },
              {
                text: "DB",
                link: "/functions/financial/db",
              },
              {
                text: "DDB",
                link: "/functions/financial/ddb",
              },
              {
                text: "DISC",
                link: "/functions/financial/disc",
              },
              {
                text: "DOLLARDE",
                link: "/functions/financial/dollarde",
              },
              {
                text: "DOLLARFR",
                link: "/functions/financial/dollarfr",
              },
              {
                text: "DURATION",
                link: "/functions/financial/duration",
              },
              {
                text: "EFFECT",
                link: "/functions/financial/effect",
              },
              {
                text: "FV",
                link: "/functions/financial/fv",
              },
              {
                text: "FVSCHEDULE",
                link: "/functions/financial/fvschedule",
              },
              {
                text: "INTRATE",
                link: "/functions/financial/intrate",
              },
              {
                text: "IPMT",
                link: "/functions/financial/ipmt",
              },
              {
                text: "IRR",
                link: "/functions/financial/irr",
              },
              {
                text: "ISPMT",
                link: "/functions/financial/ispmt",
              },
              {
                text: "MDURATION",
                link: "/functions/financial/mduration",
              },
              {
                text: "MIRR",
                link: "/functions/financial/mirr",
              },
              {
                text: "NOMINAL",
                link: "/functions/financial/nominal",
              },
              {
                text: "NPER",
                link: "/functions/financial/nper",
              },
              {
                text: "NPV",
                link: "/functions/financial/npv",
              },
              {
                text: "ODDFPRICE",
                link: "/functions/financial/oddfprice",
              },
              {
                text: "ODDFYIELD",
                link: "/functions/financial/oddfyield",
              },
              {
                text: "ODDLPRICE",
                link: "/functions/financial/oddlprice",
              },
              {
                text: "ODDLYIELD",
                link: "/functions/financial/oddlyield",
              },
              {
                text: "PDURATION",
                link: "/functions/financial/pduration",
              },
              {
                text: "PMT",
                link: "/functions/financial/pmt",
              },
              {
                text: "PPMT",
                link: "/functions/financial/ppmt",
              },
              {
                text: "PRICE",
                link: "/functions/financial/price",
              },
              {
                text: "PRICEDISC",
                link: "/functions/financial/pricedisc",
              },
              {
                text: "PRICEMAT",
                link: "/functions/financial/pricemat",
              },
              {
                text: "PV",
                link: "/functions/financial/pv",
              },
              {
                text: "RATE",
                link: "/functions/financial/rate",
              },
              {
                text: "RECEIVED",
                link: "/functions/financial/received",
              },
              {
                text: "RRI",
                link: "/functions/financial/rri",
              },
              {
                text: "SLN",
                link: "/functions/financial/sln",
              },
              {
                text: "SYD",
                link: "/functions/financial/syd",
              },
              {
                text: "TBILLEQ",
                link: "/functions/financial/tbilleq",
              },
              {
                text: "TBILLPRICE",
                link: "/functions/financial/tbillprice",
              },
              {
                text: "TBILLYIELD",
                link: "/functions/financial/tbillyield",
              },
              {
                text: "VDB",
                link: "/functions/financial/vdb",
              },
              {
                text: "XIRR",
                link: "/functions/financial/xirr",
              },
              {
                text: "XNPV",
                link: "/functions/financial/xnpv",
              },
              {
                text: "YIELD",
                link: "/functions/financial/yield",
              },
              {
                text: "YIELDDISC",
                link: "/functions/financial/yielddisc",
              },
              {
                text: "YIELDMAT",
                link: "/functions/financial/yieldmat",
              },
            ],
          },
          {
            text: "Engineering",
            collapsed: true,
            link: "/functions/engineering",
            items: [
              {
                text: "BESSELI",
                link: "/functions/engineering/besseli",
              },
              {
                text: "BESSELJ",
                link: "/functions/engineering/besselj",
              },
              {
                text: "BESSELK",
                link: "/functions/engineering/besselk",
              },
              {
                text: "BESSELY",
                link: "/functions/engineering/bessely",
              },
              {
                text: "BIN2DEC",
                link: "/functions/engineering/bin2dec",
              },
              {
                text: "BIN2HEX",
                link: "/functions/engineering/bin2hex",
              },
              {
                text: "BIN2OCT",
                link: "/functions/engineering/bin2oct",
              },
              {
                text: "BITAND",
                link: "/functions/engineering/bitand",
              },
              {
                text: "BITLSHIFT",
                link: "/functions/engineering/bitlshift",
              },
              {
                text: "BITOR",
                link: "/functions/engineering/bitor",
              },
              {
                text: "BITRSHIFT",
                link: "/functions/engineering/bitrshift",
              },
              {
                text: "BITXOR",
                link: "/functions/engineering/bitxor",
              },
              {
                text: "COMPLEX",
                link: "/functions/engineering/complex",
              },
              {
                text: "CONVERT",
                link: "/functions/engineering/convert",
              },
              {
                text: "DEC2BIN",
                link: "/functions/engineering/dec2bin",
              },
              {
                text: "DEC2HEX",
                link: "/functions/engineering/dec2hex",
              },
              {
                text: "DEC2OCT",
                link: "/functions/engineering/dec2oct",
              },
              {
                text: "DELTA",
                link: "/functions/engineering/delta",
              },
              {
                text: "ERF",
                link: "/functions/engineering/erf",
              },
              {
                text: "ERF.PRECISE",
                link: "/functions/engineering/erf-precise",
              },
              {
                text: "ERFC",
                link: "/functions/engineering/erfc",
              },
              {
                text: "ERFC.PRECISE",
                link: "/functions/engineering/erfc-precise",
              },
              {
                text: "GESTEP",
                link: "/functions/engineering/gestep",
              },
              {
                text: "HEX2BIN",
                link: "/functions/engineering/hex2bin",
              },
              {
                text: "HEX2DEC",
                link: "/functions/engineering/hex2dec",
              },
              {
                text: "HEX2OCT",
                link: "/functions/engineering/hex2oct",
              },
              {
                text: "IMABS",
                link: "/functions/engineering/imabs",
              },
              {
                text: "IMAGINARY",
                link: "/functions/engineering/imaginary",
              },
              {
                text: "IMARGUMENT",
                link: "/functions/engineering/imargument",
              },
              {
                text: "IMCONJUGATE",
                link: "/functions/engineering/imconjugate",
              },
              {
                text: "IMCOS",
                link: "/functions/engineering/imcos",
              },
              {
                text: "IMCOSH",
                link: "/functions/engineering/imcosh",
              },
              {
                text: "IMCOT",
                link: "/functions/engineering/imcot",
              },
              {
                text: "IMCSC",
                link: "/functions/engineering/imcsc",
              },
              {
                text: "IMCSCH",
                link: "/functions/engineering/imcsch",
              },
              {
                text: "IMDIV",
                link: "/functions/engineering/imdiv",
              },
              {
                text: "IMEXP",
                link: "/functions/engineering/imexp",
              },
              {
                text: "IMLN",
                link: "/functions/engineering/imln",
              },
              {
                text: "IMLOG10",
                link: "/functions/engineering/imlog10",
              },
              {
                text: "IMLOG2",
                link: "/functions/engineering/imlog2",
              },
              {
                text: "IMPOWER",
                link: "/functions/engineering/impower",
              },
              {
                text: "IMPRODUCT",
                link: "/functions/engineering/improduct",
              },
              {
                text: "IMREAL",
                link: "/functions/engineering/imreal",
              },
              {
                text: "IMSEC",
                link: "/functions/engineering/imsec",
              },
              {
                text: "IMSECH",
                link: "/functions/engineering/imsech",
              },
              {
                text: "IMSIN",
                link: "/functions/engineering/imsin",
              },
              {
                text: "IMSINH",
                link: "/functions/engineering/imsinh",
              },
              {
                text: "IMSQRT",
                link: "/functions/engineering/imsqrt",
              },
              {
                text: "IMSUB",
                link: "/functions/engineering/imsub",
              },
              {
                text: "IMSUM",
                link: "/functions/engineering/imsum",
              },
              {
                text: "IMTAN",
                link: "/functions/engineering/imtan",
              },
              {
                text: "OCT2BIN",
                link: "/functions/engineering/oct2bin",
              },
              {
                text: "OCT2DEC",
                link: "/functions/engineering/oct2dec",
              },
              {
                text: "OCT2HEX",
                link: "/functions/engineering/oct2hex",
              },
            ],
          },
          {
            text: "Database",
            collapsed: true,
            link: "/functions/database",
            items: [
              {
                text: "DAVERAGE",
                link: "/functions/database/daverage",
              },
              {
                text: "DCOUNT",
                link: "/functions/database/dcount",
              },
              {
                text: "DCOUNTA",
                link: "/functions/database/dcounta",
              },
              {
                text: "DGET",
                link: "/functions/database/dget",
              },
              {
                text: "DMAX",
                link: "/functions/database/dmax",
              },
              {
                text: "DMIN",
                link: "/functions/database/dmin",
              },
              {
                text: "DPRODUCT",
                link: "/functions/database/dproduct",
              },
              {
                text: "DSTDEV",
                link: "/functions/database/dstdev",
              },
              {
                text: "DSTDEVP",
                link: "/functions/database/dstdevp",
              },
              {
                text: "DSUM",
                link: "/functions/database/dsum",
              },
              {
                text: "DVAR",
                link: "/functions/database/dvar",
              },
              {
                text: "DVARP",
                link: "/functions/database/dvarp",
              },
            ],
          },
          {
            text: "Statistical",
            collapsed: true,
            link: "/functions/statistical",
            items: [
              {
                text: "AVEDEV",
                link: "/functions/statistical/avedev",
              },
              {
                text: "AVERAGE",
                link: "/functions/statistical/average",
              },
              {
                text: "AVERAGEA",
                link: "/functions/statistical/averagea",
              },
              {
                text: "AVERAGEIF",
                link: "/functions/statistical/averageif",
              },
              {
                text: "AVERAGEIFS",
                link: "/functions/statistical/averageifs",
              },
              {
                text: "BETA.DIST",
                link: "/functions/statistical/beta.dist",
              },
              {
                text: "BETA.INV",
                link: "/functions/statistical/beta.inv",
              },
              {
                text: "BINOM.DIST",
                link: "/functions/statistical/binom.dist",
              },
              {
                text: "BINOM.DIST.RANGE",
                link: "/functions/statistical/binom.dist.range",
              },
              {
                text: "BINOM.INV",
                link: "/functions/statistical/binom.inv",
              },
              {
                text: "CHISQ.DIST",
                link: "/functions/statistical/chisq.dist",
              },
              {
                text: "CHISQ.DIST.RT",
                link: "/functions/statistical/chisq.dist.rt",
              },
              {
                text: "CHISQ.INV",
                link: "/functions/statistical/chisq.inv",
              },
              {
                text: "CHISQ.INV.RT",
                link: "/functions/statistical/chisq.inv.rt",
              },
              {
                text: "CHISQ.TEST",
                link: "/functions/statistical/chisq.test",
              },
              {
                text: "CONFIDENCE.NORM",
                link: "/functions/statistical/confidence.norm",
              },
              {
                text: "CONFIDENCE.T",
                link: "/functions/statistical/confidence.t",
              },
              {
                text: "CORREL",
                link: "/functions/statistical/correl",
              },
              {
                text: "COUNT",
                link: "/functions/statistical/count",
              },
              {
                text: "COUNTA",
                link: "/functions/statistical/counta",
              },
              {
                text: "COUNTBLANK",
                link: "/functions/statistical/countblank",
              },
              {
                text: "COUNTIF",
                link: "/functions/statistical/countif",
              },
              {
                text: "COUNTIFS",
                link: "/functions/statistical/countifs",
              },
              {
                text: "COVARIANCE.P",
                link: "/functions/statistical/covariance.p",
              },
              {
                text: "COVARIANCE.S",
                link: "/functions/statistical/covariance.s",
              },
              {
                text: "DEVSQ",
                link: "/functions/statistical/devsq",
              },
              {
                text: "EXPON.DIST",
                link: "/functions/statistical/expon.dist",
              },
              {
                text: "F.DIST",
                link: "/functions/statistical/f.dist",
              },
              {
                text: "F.DIST.RT",
                link: "/functions/statistical/f.dist.rt",
              },
              {
                text: "F.INV",
                link: "/functions/statistical/f.inv",
              },
              {
                text: "F.INV.RT",
                link: "/functions/statistical/f.inv.rt",
              },
              {
                text: "F.TEST",
                link: "/functions/statistical/f.test",
              },
              {
                text: "FISHER",
                link: "/functions/statistical/fisher",
              },
              {
                text: "FISHERINV",
                link: "/functions/statistical/fisherinv",
              },
              {
                text: "FORECAST",
                link: "/functions/statistical/forecast",
              },
              {
                text: "FORECAST.ETS",
                link: "/functions/statistical/forecast.ets",
              },
              {
                text: "FORECAST.ETS.CONFINT",
                link: "/functions/statistical/forecast.ets.confint",
              },
              {
                text: "FORECAST.ETS.SEASONALITY",
                link: "/functions/statistical/forecast.ets.seasonality",
              },
              {
                text: "FORECAST.ETS.STAT",
                link: "/functions/statistical/forecast.ets.stat",
              },
              {
                text: "FORECAST.LINEAR",
                link: "/functions/statistical/forecast.linear",
              },
              {
                text: "FREQUENCY",
                link: "/functions/statistical/frequency",
              },
              {
                text: "GAMMA",
                link: "/functions/statistical/gamma",
              },
              {
                text: "GAMMA.DIST",
                link: "/functions/statistical/gamma.dist",
              },
              {
                text: "GAMMA.INV",
                link: "/functions/statistical/gamma.inv",
              },
              {
                text: "GAMMALN",
                link: "/functions/statistical/gammaln",
              },
              {
                text: "GAMMALN.PRECISE",
                link: "/functions/statistical/gammaln.precise",
              },
              {
                text: "GAUSS",
                link: "/functions/statistical/gauss",
              },
              {
                text: "GEOMEAN",
                link: "/functions/statistical/geomean",
              },
              {
                text: "GROWTH",
                link: "/functions/statistical/growth",
              },
              {
                text: "HARMEAN",
                link: "/functions/statistical/harmean",
              },
              {
                text: "HYPGEOM.DIST",
                link: "/functions/statistical/hypgeom.dist",
              },
              {
                text: "INTERCEPT",
                link: "/functions/statistical/intercept",
              },
              {
                text: "KURT",
                link: "/functions/statistical/kurt",
              },
              {
                text: "LARGE",
                link: "/functions/statistical/large",
              },
              {
                text: "LINEST",
                link: "/functions/statistical/linest",
              },
              {
                text: "LOGEST",
                link: "/functions/statistical/logest",
              },
              {
                text: "LOGNORM.DIST",
                link: "/functions/statistical/lognorm.dist",
              },
              {
                text: "LOGNORM.INV",
                link: "/functions/statistical/lognorm.inv",
              },
              {
                text: "MAX",
                link: "/functions/statistical/max",
              },
              {
                text: "MAXA",
                link: "/functions/statistical/maxa",
              },
              {
                text: "MAXIFS",
                link: "/functions/statistical/maxifs",
              },
              {
                text: "MEDIAN",
                link: "/functions/statistical/median",
              },
              {
                text: "MIN",
                link: "/functions/statistical/min",
              },
              {
                text: "MINA",
                link: "/functions/statistical/mina",
              },
              {
                text: "MINIFS",
                link: "/functions/statistical/minifs",
              },
              {
                text: "MODE.MULT",
                link: "/functions/statistical/mode.mult",
              },
              {
                text: "MODE.SNGL",
                link: "/functions/statistical/mode.sngl",
              },
              {
                text: "NEGBINOM.DIST",
                link: "/functions/statistical/negbinom.dist",
              },
              {
                text: "NORM.DIST",
                link: "/functions/statistical/norm.dist",
              },
              {
                text: "NORM.INV",
                link: "/functions/statistical/norm.inv",
              },
              {
                text: "NORM.S.DIST",
                link: "/functions/statistical/norm.s.dist",
              },
              {
                text: "NORM.S.INV",
                link: "/functions/statistical/norm.s.inv",
              },
              {
                text: "PEARSON",
                link: "/functions/statistical/pearson",
              },
              {
                text: "PERCENTILE.EXC",
                link: "/functions/statistical/percentile.exc",
              },
              {
                text: "PERCENTILE.INC",
                link: "/functions/statistical/percentile.inc",
              },
              {
                text: "PERCENTRANK.EXC",
                link: "/functions/statistical/percentrank.exc",
              },
              {
                text: "PERCENTRANK.INC",
                link: "/functions/statistical/percentrank.inc",
              },
              {
                text: "PERMUT",
                link: "/functions/statistical/permut",
              },
              {
                text: "PERMUTATIONA",
                link: "/functions/statistical/permutationa",
              },
              {
                text: "PHI",
                link: "/functions/statistical/phi",
              },
              {
                text: "POISSON.DIST",
                link: "/functions/statistical/poisson.dist",
              },
              {
                text: "PROB",
                link: "/functions/statistical/prob",
              },
              {
                text: "QUARTILE.EXC",
                link: "/functions/statistical/quartile.exc",
              },
              {
                text: "QUARTILE.INC",
                link: "/functions/statistical/quartile.inc",
              },
              {
                text: "RANK.AVG",
                link: "/functions/statistical/rank.avg",
              },
              {
                text: "RANK.EQ",
                link: "/functions/statistical/rank.eq",
              },
              {
                text: "RSQ",
                link: "/functions/statistical/rsq",
              },
              {
                text: "SKEW",
                link: "/functions/statistical/skew",
              },
              {
                text: "SKEW.P",
                link: "/functions/statistical/skew.p",
              },
              {
                text: "SLOPE",
                link: "/functions/statistical/slope",
              },
              {
                text: "SMALL",
                link: "/functions/statistical/small",
              },
              {
                text: "STANDARDIZE",
                link: "/functions/statistical/standardize",
              },
              {
                text: "STDEV.P",
                link: "/functions/statistical/stdev.p",
              },
              {
                text: "STDEV.S",
                link: "/functions/statistical/stdev.s",
              },
              {
                text: "STDEVA",
                link: "/functions/statistical/stdeva",
              },
              {
                text: "STDEVPA",
                link: "/functions/statistical/stdevpa",
              },
              {
                text: "STEYX",
                link: "/functions/statistical/steyx",
              },
              {
                text: "T.DIST",
                link: "/functions/statistical/t.dist",
              },
              {
                text: "T.DIST.2T",
                link: "/functions/statistical/t.dist.2t",
              },
              {
                text: "T.DIST.RT",
                link: "/functions/statistical/t.dist.rt",
              },
              {
                text: "T.INV",
                link: "/functions/statistical/t.inv",
              },
              {
                text: "T.INV.2T",
                link: "/functions/statistical/t.inv.2t",
              },
              {
                text: "T.TEST",
                link: "/functions/statistical/t.test",
              },
              {
                text: "TREND",
                link: "/functions/statistical/trend",
              },
              {
                text: "TRIMMEAN",
                link: "/functions/statistical/trimmean",
              },
              {
                text: "VAR.P",
                link: "/functions/statistical/var.p",
              },
              {
                text: "VAR.S",
                link: "/functions/statistical/var.s",
              },
              {
                text: "VARA",
                link: "/functions/statistical/vara",
              },
              {
                text: "VARPA",
                link: "/functions/statistical/varpa",
              },
              {
                text: "WEIBULL.DIST",
                link: "/functions/statistical/weibull.dist",
              },
              {
                text: "Z.TEST",
                link: "/functions/statistical/z.test",
              },
            ],
          },
          {
            text: "Text",
            collapsed: true,
            link: "/functions/text",
            items: [
              {
                text: "MID",
                link: "/functions/text/mid",
              },
              {
                text: "MIDB",
                link: "/functions/text/midb",
              },
              {
                text: "NUMBERVALUE",
                link: "/functions/text/numbervalue",
              },
              {
                text: "PHONETIC",
                link: "/functions/text/phonetic",
              },
              {
                text: "PROPER",
                link: "/functions/text/proper",
              },
              {
                text: "REPLACE",
                link: "/functions/text/replace",
              },
              {
                text: "REPLACEBS",
                link: "/functions/text/replacebs",
              },
              {
                text: "REPT",
                link: "/functions/text/rept",
              },
              {
                text: "RIGHT",
                link: "/functions/text/right",
              },
              {
                text: "RIGHTB",
                link: "/functions/text/rightb",
              },
              {
                text: "SEARCH",
                link: "/functions/text/search",
              },
              {
                text: "SEARCHB",
                link: "/functions/text/searchb",
              },
              {
                text: "SUBSTITUTE",
                link: "/functions/text/substitute",
              },
              {
                text: "T",
                link: "/functions/text/t",
              },
              {
                text: "TEXT",
                link: "/functions/text/text",
              },
              {
                text: "TEXTAFTER",
                link: "/functions/text/textafter",
              },
              {
                text: "TEXTBEFORE",
                link: "/functions/text/textbefore",
              },
              {
                text: "TEXTJOIN",
                link: "/functions/text/textjoin",
              },
              {
                text: "TEXTSPLIT",
                link: "/functions/text/textsplit",
              },
              {
                text: "TRIM",
                link: "/functions/text/trim",
              },
              {
                text: "UNICHAR",
                link: "/functions/text/unichar",
              },
              {
                text: "UNICODE",
                link: "/functions/text/unicode",
              },
              {
                text: "UPPER",
                link: "/functions/text/upper",
              },
              {
                text: "VALUE",
                link: "/functions/text/value",
              },
              {
                text: "VALUETOTEXT",
                link: "/functions/text/valuetotext",
              },
            ],
          },
          {
            text: "Math and trigonometry",
            collapsed: true,
            link: "/functions/math-and-trigonometry",
            items: [
              {
                text: "ABS",
                link: "/functions/math_and_trigonometry/abs",
              },
              {
                text: "ACOS",
                link: "/functions/math_and_trigonometry/acos",
              },
              {
                text: "ACOSH",
                link: "/functions/math_and_trigonometry/acosh",
              },
              {
                text: "ACOT",
                link: "/functions/math_and_trigonometry/acot",
              },
              {
                text: "ACOTH",
                link: "/functions/math_and_trigonometry/acoth",
              },
              {
                text: "AGGREGATE",
                link: "/functions/math_and_trigonometry/aggregate",
              },
              {
                text: "ARABIC",
                link: "/functions/math_and_trigonometry/arabic",
              },
              {
                text: "ASIN",
                link: "/functions/math_and_trigonometry/asin",
              },
              {
                text: "ASINH",
                link: "/functions/math_and_trigonometry/asinh",
              },
              {
                text: "ATAN",
                link: "/functions/math_and_trigonometry/atan",
              },
              {
                text: "ATAN2",
                link: "/functions/math_and_trigonometry/atan2",
              },
              {
                text: "ATANH",
                link: "/functions/math_and_trigonometry/atanh",
              },
              {
                text: "BASE",
                link: "/functions/math_and_trigonometry/base",
              },
              {
                text: "CEILING",
                link: "/functions/math_and_trigonometry/ceiling",
              },
              {
                text: "CEILING.MATH",
                link: "/functions/math_and_trigonometry/ceiling.math",
              },
              {
                text: "CEILING.PRECISE",
                link: "/functions/math_and_trigonometry/ceiling.precise",
              },
              {
                text: "COMBIN",
                link: "/functions/math_and_trigonometry/combin",
              },
              {
                text: "COMBINA",
                link: "/functions/math_and_trigonometry/combina",
              },
              {
                text: "COS",
                link: "/functions/math_and_trigonometry/cos",
              },
              {
                text: "COSH",
                link: "/functions/math_and_trigonometry/cosh",
              },
              {
                text: "COT",
                link: "/functions/math_and_trigonometry/cot",
              },
              {
                text: "COTH",
                link: "/functions/math_and_trigonometry/coth",
              },
              {
                text: "CSC",
                link: "/functions/math_and_trigonometry/csc",
              },
              {
                text: "CSCH",
                link: "/functions/math_and_trigonometry/csch",
              },
              {
                text: "DECIMAL",
                link: "/functions/math_and_trigonometry/decimal",
              },
              {
                text: "DEGREES",
                link: "/functions/math_and_trigonometry/degrees",
              },
              {
                text: "EVEN",
                link: "/functions/math_and_trigonometry/even",
              },
              {
                text: "EXP",
                link: "/functions/math_and_trigonometry/exp",
              },
              {
                text: "FACT",
                link: "/functions/math_and_trigonometry/fact",
              },
              {
                text: "FACTDOUBLE",
                link: "/functions/math_and_trigonometry/factdouble",
              },
              {
                text: "FLOOR",
                link: "/functions/math_and_trigonometry/floor",
              },
              {
                text: "FLOOR.MATH",
                link: "/functions/math_and_trigonometry/floor.math",
              },
              {
                text: "FLOOR.PRECISE",
                link: "/functions/math_and_trigonometry/floor.precise",
              },
              {
                text: "GCD",
                link: "/functions/math_and_trigonometry/gcd",
              },
              {
                text: "INT",
                link: "/functions/math_and_trigonometry/int",
              },
              {
                text: "ISO.CEILING",
                link: "/functions/math_and_trigonometry/iso.ceiling",
              },
              {
                text: "LCM",
                link: "/functions/math_and_trigonometry/lcm",
              },
              {
                text: "LET",
                link: "/functions/math_and_trigonometry/let",
              },
              {
                text: "LN",
                link: "/functions/math_and_trigonometry/ln",
              },
              {
                text: "LOG",
                link: "/functions/math_and_trigonometry/log",
              },
              {
                text: "LOG10",
                link: "/functions/math_and_trigonometry/log10",
              },
              {
                text: "MDETERM",
                link: "/functions/math_and_trigonometry/mdeterm",
              },
              {
                text: "MINVERSE",
                link: "/functions/math_and_trigonometry/minverse",
              },
              {
                text: "MMULT",
                link: "/functions/math_and_trigonometry/mmult",
              },
              {
                text: "MOD",
                link: "/functions/math_and_trigonometry/mod",
              },
              {
                text: "MROUND",
                link: "/functions/math_and_trigonometry/mround",
              },
              {
                text: "MULTINOMIAL",
                link: "/functions/math_and_trigonometry/multinomial",
              },
              {
                text: "MUNIT",
                link: "/functions/math_and_trigonometry/munit",
              },
              {
                text: "ODD",
                link: "/functions/math_and_trigonometry/odd",
              },
              {
                text: "PI",
                link: "/functions/math_and_trigonometry/pi",
              },
              {
                text: "POWER",
                link: "/functions/math_and_trigonometry/power",
              },
              {
                text: "PRODUCT",
                link: "/functions/math_and_trigonometry/product",
              },
              {
                text: "QUOTIENT",
                link: "/functions/math_and_trigonometry/quotient",
              },
              {
                text: "RADIANS",
                link: "/functions/math_and_trigonometry/radians",
              },
              {
                text: "RAND",
                link: "/functions/math_and_trigonometry/rand",
              },
              {
                text: "RANDARRAY",
                link: "/functions/math_and_trigonometry/randarray",
              },
              {
                text: "RANDBETWEEN",
                link: "/functions/math_and_trigonometry/randbetween",
              },
              {
                text: "ROMAN",
                link: "/functions/math_and_trigonometry/roman",
              },
              {
                text: "ROUND",
                link: "/functions/math_and_trigonometry/round",
              },
              {
                text: "ROUNDDOWN",
                link: "/functions/math_and_trigonometry/rounddown",
              },
              {
                text: "ROUNDUP",
                link: "/functions/math_and_trigonometry/roundup",
              },
              {
                text: "SEC",
                link: "/functions/math_and_trigonometry/sec",
              },
              {
                text: "SECH",
                link: "/functions/math_and_trigonometry/sech",
              },
              {
                text: "SERIESSUM",
                link: "/functions/math_and_trigonometry/seriessum",
              },
              {
                text: "SEQUENCE",
                link: "/functions/math_and_trigonometry/sequence",
              },
              {
                text: "SIGN",
                link: "/functions/math_and_trigonometry/sign",
              },
              {
                text: "SIN",
                link: "/functions/math_and_trigonometry/sin",
              },
              {
                text: "SINH",
                link: "/functions/math_and_trigonometry/sinh",
              },
              {
                text: "SQRT",
                link: "/functions/math_and_trigonometry/sqrt",
              },
              {
                text: "SQRTPI",
                link: "/functions/math_and_trigonometry/sqrtpi",
              },
              {
                text: "SUBTOTAL",
                link: "/functions/math_and_trigonometry/subtotal",
              },
              {
                text: "SUM",
                link: "/functions/math_and_trigonometry/sum",
              },
              {
                text: "SUMIF",
                link: "/functions/math_and_trigonometry/sumif",
              },
              {
                text: "SUMIFS",
                link: "/functions/math_and_trigonometry/sumifs",
              },
              {
                text: "SUMPRODUCT",
                link: "/functions/math_and_trigonometry/sumproduct",
              },
              {
                text: "SUMSQ",
                link: "/functions/math_and_trigonometry/sumsq",
              },
              {
                text: "SUMX2MY2",
                link: "/functions/math_and_trigonometry/sumx2my2",
              },
              {
                text: "SUMX2PY2",
                link: "/functions/math_and_trigonometry/sumx2py2",
              },
              {
                text: "SUMXMY2",
                link: "/functions/math_and_trigonometry/sumxmy2",
              },
              {
                text: "TAN",
                link: "/functions/math_and_trigonometry/tan",
              },
              {
                text: "TANH",
                link: "/functions/math_and_trigonometry/tanh",
              },
              {
                text: "TRUNC",
                link: "/functions/math_and_trigonometry/trunc",
              },
            ],
          },
          {
            text: "Logical",
            collapsed: true,
            link: "/functions/logical",
            items: [
              {
                text: "AND",
                link: "/functions/logical/and",
              },
              {
                text: "BYCOL",
                link: "/functions/logical/bycol",
              },
              {
                text: "BYROW",
                link: "/functions/logical/byrow",
              },
              {
                text: "FALSE",
                link: "/functions/logical/false",
              },
              {
                text: "IF",
                link: "/functions/logical/if",
              },
              {
                text: "IFERROR",
                link: "/functions/logical/iferror",
              },
              {
                text: "IFNA",
                link: "/functions/logical/ifna",
              },
              {
                text: "IFS",
                link: "/functions/logical/ifs",
              },
              {
                text: "LAMBDA",
                link: "/functions/logical/lambda",
              },
              {
                text: "LET",
                link: "/functions/logical/let",
              },
              {
                text: "MAKEARRAY",
                link: "/functions/logical/makearray",
              },
              {
                text: "MAP",
                link: "/functions/logical/map",
              },
              {
                text: "NOT",
                link: "/functions/logical/not",
              },
              {
                text: "OR",
                link: "/functions/logical/or",
              },
              {
                text: "REDUCE",
                link: "/functions/logical/reduce",
              },
              {
                text: "SCAN",
                link: "/functions/logical/scan",
              },
              {
                text: "SWITCH",
                link: "/functions/logical/switch",
              },
              {
                text: "TRUE",
                link: "/functions/logical/true",
              },
              {
                text: "XOR",
                link: "/functions/logical/xor",
              },
            ],
          },
          {
            text: "Date and time",
            collapsed: true,
            link: "/functions/date-and-time",
            items: [
              {
                text: "DATE",
                link: "/functions/date_and_time/date",
              },
              {
                text: "DATEDIF",
                link: "/functions/date_and_time/datedif",
              },
              {
                text: "DATEVALUE",
                link: "/functions/date_and_time/datevalue",
              },
              {
                text: "DAY",
                link: "/functions/date_and_time/day",
              },
              {
                text: "DAYS",
                link: "/functions/date_and_time/days",
              },
              {
                text: "DAYS360",
                link: "/functions/date_and_time/days360",
              },
              {
                text: "EDATE",
                link: "/functions/date_and_time/edate",
              },
              {
                text: "EOMONTH",
                link: "/functions/date_and_time/eomonth",
              },
              {
                text: "HOUR",
                link: "/functions/date_and_time/hour",
              },
              {
                text: "ISOWEEKNUM",
                link: "/functions/date_and_time/isoweeknum",
              },
              {
                text: "MINUTE",
                link: "/functions/date_and_time/minute",
              },
              {
                text: "MONTH",
                link: "/functions/date_and_time/month",
              },
              {
                text: "NETWORKDAYS",
                link: "/functions/date_and_time/networkdays",
              },
              {
                text: "NETWORKDAYS.INTL",
                link: "/functions/date_and_time/networkdays.intl",
              },
              {
                text: "NOW",
                link: "/functions/date_and_time/now",
              },
              {
                text: "SECOND",
                link: "/functions/date_and_time/second",
              },
              {
                text: "TIME",
                link: "/functions/date_and_time/time",
              },
              {
                text: "TIMEVALUE",
                link: "/functions/date_and_time/timevalue",
              },
              {
                text: "TODAY",
                link: "/functions/date_and_time/today",
              },
              {
                text: "WEEKDAY",
                link: "/functions/date_and_time/weekday",
              },
              {
                text: "WEEKNUM",
                link: "/functions/date_and_time/weeknum",
              },
              {
                text: "WORKDAY",
                link: "/functions/date_and_time/workday",
              },
              {
                text: "WORKDAY.INTL",
                link: "/functions/date_and_time/workday.intl",
              },
              {
                text: "YEAR",
                link: "/functions/date_and_time/year",
              },
              {
                text: "YEARFRAC",
                link: "/functions/date_and_time/yearfrac",
              },
            ],
          },
          {
            text: "Information",
            collapsed: true,
            link: "/functions/information",
            items: [
              {
                text: "CELL",
                link: "/functions/information/cell",
              },
              {
                text: "ERROR.TYPE",
                link: "/functions/information/error.type",
              },
              {
                text: "INFO",
                link: "/functions/information/info",
              },
              {
                text: "ISBLANK",
                link: "/functions/information/isblank",
              },
              {
                text: "ISERR",
                link: "/functions/information/iserr",
              },
              {
                text: "ISERROR",
                link: "/functions/information/iserror",
              },
              {
                text: "ISEVEN",
                link: "/functions/information/iseven",
              },
              {
                text: "ISFORMULA",
                link: "/functions/information/isformula",
              },
              {
                text: "ISLOGICAL",
                link: "/functions/information/islogical",
              },
              {
                text: "ISNA",
                link: "/functions/information/isna",
              },
              {
                text: "ISNONTEXT",
                link: "/functions/information/isnontext",
              },
              {
                text: "ISNUMBER",
                link: "/functions/information/isnumber",
              },
              {
                text: "ISODD",
                link: "/functions/information/isodd",
              },
              {
                text: "ISOMITTED",
                link: "/functions/information/isomitted",
              },
              {
                text: "ISREF",
                link: "/functions/information/isref",
              },
              {
                text: "ISTEXT",
                link: "/functions/information/istext",
              },
              {
                text: "N",
                link: "/functions/information/n",
              },
              {
                text: "NA",
                link: "/functions/information/na",
              },
              {
                text: "SHEET",
                link: "/functions/information/sheet",
              },
              {
                text: "SHEETS",
                link: "/functions/information/sheets",
              },
              {
                text: "TYPE",
                link: "/functions/information/type",
              },
            ],
          },
          {
            text: "Uncategorized",
            collapsed: true,
            link: "/functions/uncategorized",
            items: [
              {
                text: "REGEXTEST",
                link: "/functions/uncategorized/regextest",
              },
              {
                text: "REGEXEXTRACT",
                link: "/functions/uncategorized/regexextract",
              },
              {
                text: "REGEXREPLACE",
                link: "/functions/uncategorized/regexreplace:",
              },
              {
                text: "TRIMRANGE",
                link: "/functions/uncategorized/trimrange",
              },
            ],
          },
        ],
      },
      {
        text: "Desktop App",
        collapsed: true,
        items: [
          {
            text: "About Desktop app",
            link: "/desktop/about",
          },
        ],
      },
      {
        text: "Tironcalc",
        collapsed: true,
        items: [
          {
            text: "About Tironcalc",
            link: "/tironcalc/about",
          },
          {
            text: "Installing and basic usage",
            link: "/tironcalc/installing",
          },
        ],
      },
      {
        text: "Programming",
        collapsed: true,
        items: [
          {
            text: "About",
            link: "/programming/about",
          },
          {
            text: "Rust",
            link: "/programming/rust",
          },
          {
            text: "Python",
            link: "/programming/python-bindings",
          },
          {
            text: "JavaScript",
            link: "/programming/javascript-bindings",
          },
        ],
      },
      {
        text: "Contributing",
        collapsed: true,
        items: [
          {
            text: "How to contribute",
            link: "/contributing/how-to-contribute",
          },
        ],
      },
    ],

    editLink: {
      pattern: "https://github.com/ironcalc/ironcalc/edit/main/docs/src/:path",
      text: "Edit on GitHub",
    },

    lastUpdated: {
      text: "Updated at",
      formatOptions: {
        dateStyle: "full",
        timeStyle: "medium",
      },
    },

    socialLinks: [
      { icon: "github", link: "https://github.com/ironcalc" },
      { icon: "discord", link: "https://discord.gg/zZYWfh3RHJ" },
      { icon: "bluesky", link: "https://bsky.app/profile/ironcalc.com" },
    ],
  },
});
