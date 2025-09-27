using System;
using System.Collections.Generic;
using System.Linq;
using System.Xml;
using System.Xml.Linq;
using System.Xml.XPath;
using Newtonsoft.Json.Linq;

namespace JsonSelector.Services
{
    /// <summary>
    /// Service for parsing XML fields within JSON and applying XPath selectors
    /// </summary>
    public class XmlFieldParser
    {
        /// <summary>
        /// Cache for parsed XML documents to avoid re-parsing
        /// </summary>
        private readonly Dictionary<string, XDocument> _xmlCache = new();

        /// <summary>
        /// Parse an XML selector (e.g., "body.FpmlData@xml://trade/tradeHeader/tradeDate")
        /// </summary>
        public (string jsonPath, string xPath) ParseXmlSelector(string selector)
        {
            var xmlMarker = "@xml:";
            var xmlIndex = selector.IndexOf(xmlMarker);

            if (xmlIndex == -1)
                return (selector, null);

            var jsonPath = selector.Substring(0, xmlIndex);
            var xPath = selector.Substring(xmlIndex + xmlMarker.Length);

            return (jsonPath, xPath);
        }

        /// <summary>
        /// Check if a selector is an XML selector
        /// </summary>
        public bool IsXmlSelector(string selector)
        {
            return selector.Contains("@xml:");
        }

        /// <summary>
        /// Extract value from XML field using XPath
        /// </summary>
        public object ExtractXmlValue(string xmlContent, string xPath)
        {
            try
            {
                // Check cache first
                var cacheKey = xmlContent.GetHashCode().ToString();
                XDocument doc;

                if (!_xmlCache.TryGetValue(cacheKey, out doc))
                {
                    // Parse XML
                    doc = XDocument.Parse(xmlContent);

                    // Cache for future use (with size limit)
                    if (_xmlCache.Count < 100)
                    {
                        _xmlCache[cacheKey] = doc;
                    }
                }

                // Handle namespaces if present
                XmlNamespaceManager nsManager = null;
                if (xPath.Contains(":"))
                {
                    nsManager = GetNamespaceManager(doc);
                }

                // Evaluate XPath
                var result = nsManager != null
                    ? doc.XPathEvaluate(xPath, nsManager)
                    : doc.XPathEvaluate(xPath);

                // Convert result to appropriate type
                return ConvertXPathResult(result);
            }
            catch (Exception ex)
            {
                // Return error indication
                return $"XML_ERROR: {ex.Message}";
            }
        }

        /// <summary>
        /// Get namespace manager for FPML documents
        /// </summary>
        private XmlNamespaceManager GetNamespaceManager(XDocument doc)
        {
            var nsManager = new XmlNamespaceManager(new NameTable());

            // Add common FPML namespaces
            nsManager.AddNamespace("fpml", "http://www.fpml.org/FpML-5/confirmation");
            nsManager.AddNamespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");

            // Auto-detect namespaces from document
            var root = doc.Root;
            if (root != null)
            {
                foreach (var attr in root.Attributes())
                {
                    if (attr.IsNamespaceDeclaration)
                    {
                        var prefix = attr.Name.LocalName;
                        if (prefix != "xmlns" && !nsManager.HasNamespace(prefix))
                        {
                            nsManager.AddNamespace(prefix, attr.Value);
                        }
                    }
                }

                // Add default namespace if present
                var defaultNs = root.GetDefaultNamespace();
                if (defaultNs != null && !string.IsNullOrEmpty(defaultNs.NamespaceName))
                {
                    nsManager.AddNamespace("default", defaultNs.NamespaceName);
                }
            }

            return nsManager;
        }

        /// <summary>
        /// Convert XPath result to appropriate type
        /// </summary>
        private object ConvertXPathResult(object result)
        {
            if (result == null)
                return null;

            // Handle different XPath result types
            switch (result)
            {
                case string str:
                    return str;

                case bool b:
                    return b;

                case double d:
                    return d;

                case IEnumerable<XElement> elements:
                    var elemList = elements.ToList();
                    if (elemList.Count == 1)
                    {
                        return ExtractElementValue(elemList[0]);
                    }
                    else if (elemList.Count > 1)
                    {
                        // Return array of values for multiple elements
                        return elemList.Select(e => ExtractElementValue(e)).ToList();
                    }
                    return null;

                case IEnumerable<XAttribute> attributes:
                    var attrList = attributes.ToList();
                    if (attrList.Count == 1)
                    {
                        return attrList[0].Value;
                    }
                    else if (attrList.Count > 1)
                    {
                        return attrList.Select(a => a.Value).ToList();
                    }
                    return null;

                case XElement element:
                    return ExtractElementValue(element);

                case XAttribute attribute:
                    return attribute.Value;

                default:
                    return result.ToString();
            }
        }

        /// <summary>
        /// Extract value from XElement
        /// </summary>
        private object ExtractElementValue(XElement element)
        {
            // If element has child elements, return as structured data
            if (element.HasElements)
            {
                var result = new Dictionary<string, object>();
                foreach (var child in element.Elements())
                {
                    var key = child.Name.LocalName;
                    var value = ExtractElementValue(child);

                    // Handle multiple elements with same name
                    if (result.ContainsKey(key))
                    {
                        var existing = result[key];
                        if (existing is List<object> list)
                        {
                            list.Add(value);
                        }
                        else
                        {
                            result[key] = new List<object> { existing, value };
                        }
                    }
                    else
                    {
                        result[key] = value;
                    }
                }

                // Add attributes
                foreach (var attr in element.Attributes())
                {
                    if (!attr.IsNamespaceDeclaration)
                    {
                        result["@" + attr.Name.LocalName] = attr.Value;
                    }
                }

                return result;
            }

            // Return element value
            return element.Value;
        }

        /// <summary>
        /// Clear XML cache
        /// </summary>
        public void ClearCache()
        {
            _xmlCache.Clear();
        }
    }
}