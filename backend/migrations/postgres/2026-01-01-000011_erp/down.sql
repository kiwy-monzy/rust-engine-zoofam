-- ERP migration rollback
DROP TABLE IF EXISTS erp_expenses CASCADE;
DROP TABLE IF EXISTS erp_assets CASCADE;
DROP TABLE IF EXISTS erp_shipments CASCADE;
DROP TABLE IF EXISTS erp_sales_lines CASCADE;
DROP TABLE IF EXISTS erp_sales_orders CASCADE;
DROP TABLE IF EXISTS erp_inventory_transactions CASCADE;
DROP TABLE IF EXISTS erp_goods_receipt_lines CASCADE;
DROP TABLE IF EXISTS erp_goods_receipts CASCADE;
DROP TABLE IF EXISTS erp_purchase_order_lines CASCADE;
DROP TABLE IF EXISTS erp_purchase_orders CASCADE;
DROP TABLE IF EXISTS erp_procurement_lines CASCADE;
DROP TABLE IF EXISTS erp_procurement_requests CASCADE;
DROP TABLE IF EXISTS erp_products CASCADE;
DROP TABLE IF EXISTS erp_suppliers CASCADE;
DROP TABLE IF EXISTS erp_warehouses CASCADE;
DROP TABLE IF EXISTS erp_units CASCADE;
